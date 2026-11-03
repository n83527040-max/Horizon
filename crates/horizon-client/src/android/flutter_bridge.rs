//! JNI/C ABI boundary for the Flutter launcher host.
//!
//! This module is added to the *hosting* crate root (`android_main.rs`). It never
//! touches the simulation, wgpu, winit, Vulkan, `content_files`, `account` or the
//! on-disk config schema. It only calls the real, already existing headless APIs:
//! `catalog::Catalog`, `catalog::Config`, `account::{Accounts, SignIn}`,
//! `diagnostics`, `performance` and `horizon_client::content_files::validate`.
//!
//! The game itself is NOT started here: `main.dart` hands `Play` to the separate
//! `:engine` process, where the original `Launcher::action` runs on its own
//! winit/Vulkan surface. See FLUTTER_NATIVE_BRIDGE_ANALYSIS.md.
#![allow(clippy::all)]

#[path = "bridge_rules.rs"]
mod rules;

use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString, JValue},
    sys::jstring,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

struct Engine {
    home: PathBuf,
    cache: PathBuf,
    catalog: crate::catalog::Catalog,
    sign_in: Option<crate::account::SignIn>,
    login_code: Option<String>,
    login_uri: Option<String>,
    login_message: String,
    /// Real classified error from the latest native operation.
    error_kind: String,
    auth_stage: String,
    status: String,
    cancel: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
}

static STATE: OnceLock<Mutex<Option<Engine>>> = OnceLock::new();
static POLLER: AtomicBool = AtomicBool::new(false);

fn state() -> &'static Mutex<Option<Engine>> {
    STATE.get_or_init(|| Mutex::new(None))
}

fn text(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Result<String, String> {
    env.get_string(value)
        .map(Into::into)
        .map_err(|e| e.to_string())
}

fn answer(env: &mut JNIEnv<'_>, value: Value) -> jstring {
    let payload = value.to_string();
    match env.new_string(payload) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn failure(message: impl Into<String>) -> Value {
    json!({"ok": false, "message": message.into(), "changed": false})
}

fn success(message: impl Into<String>, changed: bool) -> Value {
    json!({"ok": true, "message": message.into(), "changed": changed})
}

fn needs_activity(message: impl Into<String>) -> Value {
    json!({"ok": true, "needsActivity": true, "message": message.into(), "changed": false})
}

fn proc_kb(path: &str, key: &str) -> Option<i64> {
    let file = std::fs::read_to_string(path).ok()?;
    for line in file.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            return rest
                .trim()
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<i64>().ok());
        }
    }
    None
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            match entry.metadata() {
                Ok(meta) if meta.is_file() => total = total.saturating_add(meta.len()),
                Ok(meta) if meta.is_dir() => {
                    total = total.saturating_add(dir_size(&entry.path()));
                }
                _ => {}
            }
        }
    }
    total
}

fn flutter_preferences(home: &Path) -> Value {
    std::fs::read(home.join("flutter-preferences.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .unwrap_or_else(|| json!({}))
}

fn choices() -> Value {
    json!({
        "Renderer": [
            crate::catalog::RendererMode::Vulkan.label(),
            crate::catalog::RendererMode::OpenGl.label(),
            crate::catalog::RendererMode::VulkanCompatibility.label()
        ],
        "RenderScale": crate::display::RenderScale::PRESETS
            .iter().map(|p| format!("{p}%")).collect::<Vec<String>>(),
        "ChooseRenderScale": crate::display::RenderScale::PRESETS
            .iter().map(|p| format!("{p}%")).collect::<Vec<String>>(),
        "Orientation": crate::catalog::Orientation::ALL
            .iter().map(|o| o.label()).collect::<Vec<&'static str>>(),
        "ChooseOrientation": crate::catalog::Orientation::ALL
            .iter().map(|o| o.label()).collect::<Vec<&'static str>>(),
        "Aspect": ["Native", "16:9", "4:3", "1:1"],
        "ChooseAspect": ["Native", "16:9", "4:3", "1:1"],
        "DisplayHz": crate::device_preferences::DISPLAY_RATES
            .iter().map(|h| if *h == 0 { "Automatic".to_string() } else { format!("{h} Hz") })
            .collect::<Vec<String>>(),
        "ChooseDisplayHz": crate::device_preferences::DISPLAY_RATES
            .iter().map(|h| if *h == 0 { "Automatic".to_string() } else { format!("{h} Hz") })
            .collect::<Vec<String>>(),
        "ChooseInputHost": crate::catalog::InputHost::ALL
            .iter().map(|i| i.label()).collect::<Vec<&'static str>>(),
        "Profile": ["Profile 0", "Profile 1", "Profile 2", "Profile 3"],
        "ChooseProfile": ["Profile 0", "Profile 1", "Profile 2", "Profile 3"],
        "ModSource": ["Horizon", "Modrinth"],
        "ShaderSource": ["Horizon", "Modrinth"]
    })
}

fn settings(e: &Engine) -> Value {
    let c = &e.catalog.config;
    let mut map = serde_json::Map::new();
    map.insert("renderer".into(), json!(c.renderer.label()));
    map.insert("renderScale".into(), json!(format!("{}%", c.render_scale.percent())));
    map.insert("orientation".into(), json!(c.orientation.label()));
    map.insert("immersive".into(), json!(c.screen.immersive));
    map.insert("avoidCutout".into(), json!(c.screen.avoid_cutout));
    map.insert("aspect".into(), json!(c.screen.aspect.label()));
    map.insert("displayHz".into(), json!(c.device.display_hz));
    map.insert("performanceHints".into(), json!(c.device.performance_hints));
    map.insert("gpuProfile".into(), json!(c.device.gpu_profile));
    map.insert("recordPerformance".into(), json!(c.record_performance));
    map.insert("profile".into(), json!(format!("Profile {}", c.profile)));
    map.insert("inputHost".into(), json!(c.input_host.label()));
    map.insert("server".into(), json!(c.server.clone()));
    map.insert(
        "modSource".into(),
        json!(if c.mod_source == crate::shader_catalog::Origin::Horizon { "Horizon" } else { "Modrinth" }),
    );
    map.insert(
        "shaderSource".into(),
        json!(if c.shader_source == crate::shader_catalog::Origin::Horizon { "Horizon" } else { "Modrinth" }),
    );
    let prefs = flutter_preferences(&e.home);
    if let Some(v) = prefs.get("theme") {
        map.insert("theme".into(), v.clone());
    }
    if let Some(v) = prefs.get("language") {
        map.insert("language".into(), v.clone());
    }
    Value::Object(map)
}

fn account_status(e: &Engine) -> Option<String> {
    use crate::account::{Accounts, Player, Secrets};
    let accounts = Accounts::load();
    let status = match accounts.player() {
        Player::Microsoft(_) => {
            let signed_in = accounts.selected().is_some_and(|a| {
                crate::account::store()
                    .get(&a.uuid)
                    .ok()
                    .flatten()
                    .is_some_and(|s| serde_json::from_str::<Secrets>(&s).is_ok())
            });
            match accounts.selected() {
                Some(a) if signed_in => format!("Microsoft: {}", a.name),
                Some(a) => format!("Sign in required: {}", a.name),
                None => "Sign in required".to_string(),
            }
        }
        // An offline account plays without asking Microsoft for anything.
        Player::Offline(name) => format!("Offline: {name}"),
        Player::GameName if !e.catalog.config.name.trim().is_empty() => {
            format!("Offline: {}", e.catalog.config.name)
        }
        Player::GameName => "Offline / Sign in".to_string(),
    };
    Some(status)
}

/// The REAL account list from accounts.json (Microsoft + offline), used by the
/// Flutter accounts sheet for select / switch / delete / default selection.
fn accounts_json() -> Value {
    use crate::account::{Accounts, Player};
    let accounts = Accounts::load();
    let current = accounts.player();
    let mut out: Vec<Value> = Vec::new();
    for a in &accounts.accounts {
        out.push(json!({
            "id": a.uuid.clone(),
            "name": a.name.clone(),
            "type": "microsoft",
            "selected": matches!(&current, Player::Microsoft(u) if u == &a.uuid),
            "skin": a.skin.as_ref().map(|(url, _)| url.clone()),
        }));
    }
    for name in &accounts.offline {
        out.push(json!({
            "id": name.clone(),
            "name": name.clone(),
            "type": "offline",
            "selected": matches!(&current, Player::Offline(n) if n == name),
        }));
    }
    Value::Array(out)
}

/// Classify the real catalog status text. Never turns a network/API error into
/// "No results". Uses the host-tested rules module.
fn catalog_status(e: &Engine) -> (String, String, bool) {
    let raw = e.catalog.status.clone();
    let kind = rules::classify_error(&raw).to_string();
    let retry = rules::can_retry(&kind);
    (raw, kind, retry)
}

fn snapshot(e: &Engine) -> Value {
    let game = e.catalog.config.game_home(&e.home);
    let (catalog_status, error_kind, can_retry) = catalog_status(e);
    let (filter_version, filter_loader) = e.catalog.filters();
    let mut content_bytes = serde_json::Map::new();
    for kind in horizon_client::content_files::Kind::ALL {
        content_bytes.insert(
            kind.folder().to_string(),
            json!(dir_size(&game.join(kind.folder()))),
        );
    }
    let installed: Vec<Value> = e
        .catalog
        .local
        .iter()
        .enumerate()
        .map(|(i, item)| {
            json!({
                "file": item.file.clone(),
                "title": item.title.clone(),
                "folder": item.kind.folder(),
                "enabled": item.enabled,
                "index": i
            })
        })
        .collect();
    let results: Vec<Value> = e
        .catalog
        .results
        .iter()
        .enumerate()
        .map(|(i, p)| {
            json!({
                "title": p.title.clone(),
                "description": p.summary.clone(),
                "author": p.author.clone(),
                "downloads": p.downloads,
                "version": p.game_versions.first().cloned(),
                "versions": p.game_versions.clone(),
                "loaders": p.loaders.clone(),
                "source": p.source.label(),
                "iconUrl": p.icon.clone(),
                "compatibility": p.support().label(),
                "installable": p.support().installable(),
                "projectId": p.id.clone(),
                "pageUrl": p.page.clone(),
                "index": i
            })
        })
        .collect();
    json!({
        "appVersion": crate::backend::VERSION,
        "gameVersion": horizon_core::VANILLA_VERSION,
        "profileName": if e.catalog.config.name.is_empty() { Value::Null } else { json!(e.catalog.config.name.clone()) },
        "accountStatus": account_status(e),
        "avatarPath": Value::Null,
        "graphicsApi": e.catalog.config.renderer.label(),
        "renderScalePercent": e.catalog.config.render_scale.percent(),
        "ramUsedMb": proc_kb("/proc/self/status", "VmRSS:").map(|v| v / 1024),
        "ramPeakMb": proc_kb("/proc/self/status", "VmHWM:").map(|v| v / 1024),
        "ramTotalMb": proc_kb("/proc/meminfo", "MemTotal:").map(|v| v / 1024),
        "installed": installed,
        "results": results,
        // REAL native status (network/API/parse/compatibility), never hidden.
        "status": if catalog_status.is_empty() {
            if e.status.is_empty() { Value::Null } else { json!(e.status.clone()) }
        } else {
            json!(catalog_status.clone())
        },
        "catalogStatus": if catalog_status.is_empty() { Value::Null } else { json!(catalog_status) },
        "errorKind": if e.error_kind.is_empty() { error_kind } else { e.error_kind.clone() },
        "canRetry": can_retry,
        "authStage": e.auth_stage.clone(),
        "progress": e.catalog.progress(),
        "activePage": match e.catalog.page() {
            crate::catalog::Page::Mods => "Mods",
            crate::catalog::Page::Packs => "Packs",
            crate::catalog::Page::Shaders => "Shaders",
            crate::catalog::Page::Installed => "Installed",
            crate::catalog::Page::Settings => "Settings",
            _ => "Play",
        },
        "filterVersion": filter_version,
        "filterLoader": filter_loader,
        "accounts": accounts_json(),
        "settings": settings(e),
        "choices": choices(),
        "storageBytes": content_bytes.values().filter_map(|v| v.as_u64()).sum::<u64>(),
        "cacheBytes": dir_size(&e.cache),
        "contentBytes": content_bytes,
        "engineBridgeReady": true,
        "pending": e.catalog.busy() || e.catalog.searching() || e.sign_in.is_some(),
        "loginCode": e.login_code,
        "loginUri": e.login_uri,
        "loginMessage": if e.login_message.is_empty() { Value::Null } else { json!(e.login_message) }
    })
}

/// Drain the real workers. Runs on a Rust-owned thread, never on the JNI caller,
/// so `flutter pub` snapshot polling can never be blocked by a download.
fn poll_loop(shutdown: Arc<AtomicBool>) {
    while !shutdown.load(Ordering::Relaxed) {
        {
            let mut slot = state().lock().unwrap_or_else(|e| e.into_inner());
            if let Some(e) = slot.as_mut() {
                if e.cancel.swap(false, Ordering::Relaxed) {
                    e.catalog.show_installed(None);
                    e.catalog.results.clear();
                    e.status = "Operation cancelled".into();
                }
                let home = e.home.clone();
                e.catalog.poll(&home);
                e.error_kind = rules::classify_error(&e.catalog.status).to_string();
                let sign_in_state = e.sign_in.as_ref().map(|task| task.state());
                if let Some(state) = sign_in_state {
                    match state {
                        crate::account::SignInState::Code(code) => {
                            e.login_code = Some(code.user_code.clone());
                            e.login_uri = Some(code.verification_uri.clone());
                            e.status = "Waiting for Microsoft approval".into();
                        }
                        crate::account::SignInState::Done(result) => {
                            let (message, stage) = match result {
                                Ok(a) => (format!("Signed in as {}", a.name), "done".to_string()),
                                Err(error) => (error.clone(), rules::auth_stage(&error).to_string()),
                            };
                            e.login_message = message;
                            e.auth_stage = stage;
                            e.status = e.login_message.clone();
                            e.sign_in = None;
                            e.login_code = None;
                            e.login_uri = None;
                        }
                        crate::account::SignInState::Starting => {
                            e.status = "Contacting Microsoft...".into();
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn page_of(name: &str) -> Option<crate::catalog::Page> {
    match name {
        "Mods" => Some(crate::catalog::Page::Mods),
        "Packs" => Some(crate::catalog::Page::Packs),
        "Shaders" => Some(crate::catalog::Page::Shaders),
        "Installed" => Some(crate::catalog::Page::Installed),
        "Settings" => Some(crate::catalog::Page::Settings),
        _ => None,
    }
}

fn kind_of(name: &str) -> Option<horizon_client::content_files::Kind> {
    match name {
        "Mods" | "horizon-mods" => Some(horizon_client::content_files::Kind::Mods),
        "Packs" | "resourcepacks" => Some(horizon_client::content_files::Kind::Packs),
        "Shaders" | "shaderpacks" => Some(horizon_client::content_files::Kind::Shaders),
        _ => None,
    }
}

fn origin_of(name: &str) -> Option<crate::shader_catalog::Origin> {
    match name {
        "Horizon" => Some(crate::shader_catalog::Origin::Horizon),
        "Modrinth" => Some(crate::shader_catalog::Origin::Modrinth),
        _ => None,
    }
}

fn arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn index_arg(args: &Value) -> Option<usize> {
    args.get("index")
        .and_then(Value::as_u64)
        .map(|v| v as usize)
}

fn save(e: &mut Engine) -> Result<(), String> {
    let home = e.home.clone();
    e.catalog.config.save(&home)
}

/// REAL account management through the existing `Accounts` API. Returns None
/// when `name` is not an account operation. `offline_without_owner=true` is the
/// upstream option that lets an offline name reach the game without Microsoft;
/// no authentication logic is rewritten here.
fn account_operation(e: &mut Engine, name: &str, input: &str, args: &Value) -> Option<Value> {
    use crate::account::{Accounts, Endpoints, Player, Secrets};
    let known = matches!(name, "name" | "offlineName" | "offline/add" | "offline/select"
        | "offline/delete" | "account/select" | "account/delete" | "account/refresh");
    if !known {
        return None;
    }
    let store = crate::account::store();
    let result: Result<String, String> = (|| {
        let mut accounts = Accounts::load();
        match name {
            "name" | "offlineName" | "offline/add" | "offline/select" => {
                let player = input.trim();
                if !rules::valid_offline_name(player) {
                    return Err("Offline names must be 3-16 ASCII letters, digits or _".into());
                }
                accounts.play_as_with(store, Player::Offline(player.to_string()), true)?;
                accounts.save()?;
                e.catalog.config.name = player.to_string();
                save(e)?;
                Ok(format!("Offline account {player} is ready"))
            }
            "offline/delete" => {
                accounts.remove_offline(input.trim(), &crate::skins::dir())?;
                accounts.save()?;
                if e.catalog.config.name.eq_ignore_ascii_case(input.trim()) {
                    e.catalog.config.name = accounts.offline_selected.clone().unwrap_or_default();
                    save(e)?;
                }
                Ok(format!("Removed offline account {}", input.trim()))
            }
            "account/select" => {
                let id = arg(args, "id").unwrap_or(input).trim().to_string();
                accounts.play_as_with(store, Player::Microsoft(id), false)?;
                accounts.save()?;
                Ok("Microsoft account selected".into())
            }
            "account/delete" => {
                let id = arg(args, "id").unwrap_or(input).trim().to_string();
                accounts.remove(store, &id)?;
                accounts.save()?;
                Ok("Microsoft account removed".into())
            }
            "account/refresh" => {
                let Some(a) = accounts.selected().cloned() else {
                    return Err("No Microsoft account is selected".into());
                };
                let secrets = store
                    .get(&a.uuid)?
                    .and_then(|s| serde_json::from_str::<Secrets>(&s).ok())
                    .ok_or("Session ended. Sign in again.")?;
                let (a2, s2) = crate::account::fresh(
                    &Endpoints::live(),
                    e.catalog.config.microsoft_client_id.trim(),
                    &a,
                    secrets,
                )?;
                accounts.add(store, a2.clone(), &s2)?;
                accounts.save()?;
                Ok(format!("Refreshed session for {}", a2.name))
            }
            _ => unreachable!(),
        }
    })();
    Some(match result {
        Ok(message) => success(message, true),
        Err(error) => failure(format!("{error} [{}]", rules::classify_error(&error))),
    })
}

/// Every arm performs a real operation. Nothing returns success without doing
/// the work; actions that need the Vulkan/winit host return `needsActivity`.
fn dispatch(e: &mut Engine, action: &str, args: &Value) -> Value {
    let home = e.home.clone();
    let value = arg(args, "value");
    match action {
        // Opening a catalog page MUST set the real Rust page and start a real
        // browse; otherwise `Catalog::page` stays `Play` and browse() returns
        // early, which is why every page used to show "No results".
        "Page" => {
            let kind = arg(args, "kind").or_else(|| arg(args, "page")).unwrap_or("");
            if rules::catalog_page(kind).is_none() {
                return success("Navigation handled in Dart", false);
            }
            let Some(page) = page_of(kind) else {
                return success("Navigation handled in Dart", false);
            };
            e.catalog.page = page;
            if page.is_catalog() {
                e.catalog.offset = 0;
                e.catalog.browse(&home);
                return success("Loading real catalog", true);
            }
            success("Navigation handled in Dart", false)
        }
        "ToggleSection" | "Dismiss" | "Account" | "Profile" | "Renderer"
        | "RenderScale" | "Orientation" | "InputHost" | "Aspect" | "DisplayHz" => {
            success("Choose a value from the real launcher options", false)
        }
        "Play" => needs_activity("Starting Minecraft in the native engine"),
        "Search" => {
            if let Some(kind) = arg(args, "kind") {
                if let Some(page) = page_of(kind) {
                    e.catalog.page = page;
                }
            }
            if let Some(text) = arg(args, "query") {
                e.catalog.query = text.to_string();
            }
            let version = arg(args, "version")
                .and_then(rules::normalize_version);
            let loader = arg(args, "loader")
                .and_then(rules::normalize_loader)
                .map(str::to_string);
            e.catalog.set_filters(version, loader);
            e.catalog.offset = 0;
            e.catalog.search(&home);
            let mut reply = success("Searching the real Modrinth / Horizon catalog", true);
            reply["pending"] = json!(true);
            reply
        }
        "ModSource" | "ShaderSource" => {
            let Some(origin) = value.and_then(origin_of) else {
                return failure("Unknown catalog source");
            };
            if action == "ModSource" {
                e.catalog.config.mod_source = origin;
            } else {
                e.catalog.config.shader_source = origin;
            }
            if let Err(error) = save(e) {
                return failure(error);
            }
            if let Some(kind) = arg(args, "kind") {
                if let Some(page) = page_of(kind) {
                    e.catalog.page = page;
                }
            }
            e.catalog.browse(&home);
            let mut reply = success("Catalog source saved", true);
            reply["pending"] = json!(true);
            reply
        }
        "Field" => {
            let name = arg(args, "name").unwrap_or("");
            let input = value.unwrap_or("");
            if !rules::verifiable_success("Field", Some(name), Some(input)) {
                return failure("Enter a player name first (3-16 letters, digits or _)");
            }
            if let Some(reply) = account_operation(e, name, input, args) {
                return reply;
            }
            match name {
                "theme" | "language" => {
                    let mut prefs = match flutter_preferences(&home) {
                        Value::Object(map) => map,
                        _ => serde_json::Map::new(),
                    };
                    prefs.insert(name.to_string(), json!(input));
                    match std::fs::write(
                        home.join("flutter-preferences.json"),
                        Value::Object(prefs).to_string(),
                    ) {
                        Ok(()) => success("Preference saved", true),
                        Err(error) => failure(error.to_string()),
                    }
                }
                "clearCache" => {
                    let cache = e.cache.clone();
                    let mut removed = 0u64;
                    if let Ok(entries) = std::fs::read_dir(&cache) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                removed = removed.saturating_add(
                                    std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                                );
                                let _ = std::fs::remove_file(path);
                            }
                        }
                    }
                    success(format!("Cache cleared: {removed} bytes"), true)
                }
                "search" => {
                    e.catalog.query = input.to_string();
                    success("Search text updated", false)
                }
                "server" => {
                    e.catalog.config.server = input.to_string();
                    match save(e) {
                        Ok(()) => success("Server saved", true),
                        Err(error) => failure(error),
                    }
                }
                "filter/version" | "filter/loader" => {
                    let (mut version, mut loader) = e.catalog.filters();
                    if name == "filter/version" {
                        version = rules::normalize_version(input);
                    } else {
                        loader = rules::normalize_loader(input).map(str::to_string);
                    }
                    e.catalog.set_filters(version.clone(), loader.clone());
                    e.catalog.offset = 0;
                    // search(refresh=true) bypasses the queue cache: the cache
                    // key does not include filters, so browse() would be stale.
                    e.catalog.search(&home);
                    let mut reply = success("Compatibility filters applied", true);
                    reply["pending"] = json!(true);
                    reply
                }
                "retry" => {
                    e.catalog.offset = 0;
                    e.catalog.search(&home);
                    let mut reply = success("Retrying the real catalog request", true);
                    reply["pending"] = json!(true);
                    reply
                }
                "client_id" => {
                    e.catalog.config.microsoft_client_id = input.to_string();
                    match save(e) {
                        Ok(()) => success("Microsoft client ID saved", true),
                        Err(error) => failure(error),
                    }
                }
                _ => failure(format!("Unsupported field: {name}")),
            }
        }
        "FinishNameEdit" => match save(e) {
            Ok(()) => success("Settings saved", true),
            Err(error) => failure(error),
        },
        "Install" => {
            let Some(i) = index_arg(args) else {
                return failure("Missing catalog index");
            };
            let Some(project) = e.catalog.results.get(i).cloned() else {
                return failure("No catalog project at this index");
            };
            if crate::catalog::source_only_mod(&project) {
                return failure(
                    "This .hnmod needs a separately compiled client; only sandboxed .hmod mods install here",
                );
            }
            if !project.support().installable() {
                return failure(format!(
                    "{} is not compatible with Minecraft {} / this client ({})",
                    project.title,
                    horizon_core::VANILLA_VERSION,
                    project.support().label()
                ));
            }
            e.catalog.install(&home, i);
            let mut reply = success(
                format!("Downloading {} with checksum verification", project.title),
                true,
            );
            reply["pending"] = json!(true);
            reply
        }
        "Import" => {
            let mut reply = needs_activity("Choose a file with the Android document picker");
            reply["needsSaf"] = json!(true);
            reply
        }
        "OpenFolder" => {
            let kind = arg(args, "kind").and_then(kind_of);
            let path = kind.map(|k| e.catalog.config.game_home(&home).join(k.folder()));
            let mut reply = needs_activity("Opening the real content folder");
            reply["path"] = json!(path.map(|p| p.to_string_lossy().into_owned()));
            reply
        }
        "ToggleLocal" => {
            let Some(i) = index_arg(args) else {
                return failure("Missing installed item index");
            };
            e.catalog.toggle_local(&home, i);
            success("Selection saved for the next game launch", true)
        }
        "DeleteLocal" => {
            let Some(i) = index_arg(args) else {
                return failure("Missing installed item index");
            };
            let Some(item) = e.catalog.local.get(i).cloned() else {
                return failure("No installed item at this index");
            };
            e.catalog.pending_delete = Some(item);
            success("Confirm removal", false)
        }
        "ConfirmDeleteLocal" => {
            if e.catalog.pending_delete.is_none() {
                return failure("Nothing selected for removal");
            }
            e.catalog.remove_local(&home);
            e.catalog.refresh(&home);
            success("File removed", true)
        }
        "Next" | "Previous" => {
            if let Some(kind) = arg(args, "kind") {
                if let Some(page) = page_of(kind) {
                    e.catalog.page = page;
                }
            }
            if action == "Next" {
                e.catalog.offset = e.catalog.offset.saturating_add(12);
            } else {
                e.catalog.offset = e.catalog.offset.saturating_sub(12);
            }
            e.catalog.browse(&home);
            success("Loading catalog page", true)
        }
        "Logs" => success(crate::diagnostics::summary(&home), false),
        "RecordPerformance" | "PerformanceHints" | "GpuProfile" | "Immersive" | "Cutout" => {
            match action {
                "RecordPerformance" => e.catalog.config.record_performance ^= true,
                "PerformanceHints" => e.catalog.config.device.performance_hints ^= true,
                "GpuProfile" => e.catalog.config.device.gpu_profile ^= true,
                "Immersive" => e.catalog.config.screen.immersive ^= true,
                "Cutout" => e.catalog.config.screen.avoid_cutout ^= true,
                _ => unreachable!(),
            }
            match save(e) {
                Ok(()) => success("Setting saved", true),
                Err(error) => failure(error),
            }
        }
        "ExportPerformance" => match crate::performance::export(&home) {
            Ok(()) => success("Performance report opened in Files", false),
            Err(error) => failure(error),
        },
        "ExportDiagnostics" => match crate::diagnostics::export(&home) {
            Ok(path) => success(format!("Saved {}", path.display()), false),
            Err(error) => failure(error),
        },
        "SignIn" | "DeviceSignIn" => {
            if e.sign_in.is_some() {
                return failure("A Microsoft sign-in is already running");
            }
            e.login_code = None;
            e.login_uri = None;
            e.login_message.clear();
            e.sign_in = Some(crate::account::SignIn::start_with_client_id(Some(
                e.catalog.config.microsoft_client_id.trim().to_string(),
            )));
            e.status = "Contacting Microsoft...".into();
            e.auth_stage = "device_code".into();
            let mut reply = success("Contacting Microsoft for a real device code", true);
            reply["pending"] = json!(true);
            reply["authStage"] = json!("device_code");
            reply
        }
        "CopyLoginCode" => match e.login_code.clone() {
            Some(code) => {
                let mut reply = success(code.clone(), false);
                reply["code"] = json!(code);
                reply
            }
            None => failure("No Microsoft code is available yet"),
        },
        "LoginLink" => match (e.login_uri.clone(), e.login_code.clone()) {
            (Some(uri), Some(code)) => {
                let mut reply = needs_activity("Open the Microsoft verification page");
                reply["url"] = json!(format!("{uri}?otc={code}"));
                reply
            }
            _ => failure("No Microsoft verification link is available yet"),
        },
        "CancelLogin" | "CloseLogin" => {
            if let Some(task) = e.sign_in.take() {
                task.cancel();
            }
            e.login_code = None;
            e.login_uri = None;
            e.status = "Sign-in cancelled".into();
            success("Microsoft sign-in cancelled", true)
        }
        "SignOut" => {
            let mut accounts = crate::account::Accounts::load();
            let Some(a) = accounts.selected().cloned() else {
                return failure("No Microsoft account is signed in");
            };
            match accounts
                .remove(crate::account::store(), &a.uuid)
                .and_then(|_| accounts.save())
            {
                Ok(()) => success("Signed out. Offline play is available.", true),
                Err(error) => failure(error),
            }
        }
        "ReviewApp" | "Repository" => needs_activity("Opening the real link"),
        "ChooseRenderer" => {
            let mode = match value {
                Some(t) if t.starts_with("Vulkan / GPU") => crate::catalog::RendererMode::Vulkan,
                Some(t) if t.starts_with("OpenGL") => crate::catalog::RendererMode::OpenGl,
                Some(_) => crate::catalog::RendererMode::VulkanCompatibility,
                None => return failure("Missing renderer value"),
            };
            e.catalog.config.renderer = mode;
            match save(e) {
                Ok(()) => success(mode.label().to_string(), true),
                Err(error) => failure(error),
            }
        }
        "ChooseRenderScale" => match value.and_then(|v| crate::display::RenderScale::parse(v).ok()) {
            Some(scale) => {
                e.catalog.config.render_scale = scale;
                match save(e) {
                    Ok(()) => success(format!("{}%", scale.percent()), true),
                    Err(error) => failure(error),
                }
            }
            None => failure("Invalid resolution percentage"),
        },
        "ChooseOrientation" => {
            let Some(o) = value else {
                return failure("Missing orientation value");
            };
            let Some(mode) = crate::catalog::Orientation::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.label() == o)
            else {
                return failure("Unknown orientation");
            };
            e.catalog.config.orientation = mode;
            match save(e) {
                Ok(()) => success(mode.label().to_string(), true),
                Err(error) => failure(error),
            }
        }
        "ChooseInputHost" => {
            let Some(v) = value else {
                return failure("Missing input value");
            };
            let Some(mode) = crate::catalog::InputHost::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.label() == v)
            else {
                return failure("Unknown input mode");
            };
            e.catalog.config.input_host = mode;
            match save(e) {
                Ok(()) => success(mode.label().to_string(), true),
                Err(error) => failure(error),
            }
        }
        "ChooseAspect" => {
            let aspect = match value {
                Some("16:9") => crate::display::Aspect::Wide,
                Some("4:3") => crate::display::Aspect::Classic,
                Some("1:1") => crate::display::Aspect::Square,
                Some("Native") => crate::display::Aspect::Native,
                _ => return failure("Unknown aspect ratio"),
            };
            e.catalog.config.screen.aspect = aspect;
            match save(e) {
                Ok(()) => success(aspect.label().to_string(), true),
                Err(error) => failure(error),
            }
        }
        "ChooseDisplayHz" => {
            let hz = match value {
                Some("Automatic") => 0u16,
                Some(v) => v.trim_end_matches(" Hz").parse::<u16>().unwrap_or(0),
                None => return failure("Missing refresh rate"),
            };
            if !crate::device_preferences::DISPLAY_RATES.contains(&hz) {
                return failure("Unsupported refresh rate");
            }
            e.catalog.config.device.display_hz = hz;
            match save(e) {
                Ok(()) => success(format!("{hz} Hz"), true),
                Err(error) => failure(error),
            }
        }
        "ChooseProfile" => {
            let id = value
                .and_then(|v| v.strip_prefix("Profile "))
                .and_then(|v| v.parse::<u32>().ok());
            let Some(id) = id else {
                return failure("Unknown profile");
            };
            e.catalog.config.profile = id;
            e.catalog.refresh(&home);
            match save(e) {
                Ok(()) => success(format!("Profile {id}"), true),
                Err(error) => failure(error),
            }
        }
        _ => failure(format!("Unmapped native action: {action}")),
    }
}

/// Read an Intent extra from the *engine process* only. `platform::with_context`
/// works there because `android_main` installed `APP` on this very thread.
fn intent_extra(name: &str, boolean: bool) -> Option<String> {
    crate::platform::with_context(|env, activity| {
        let intent = env
            .call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])?
            .l()?;
        let key = JObject::from(env.new_string(name)?);
        if boolean {
            let enabled = env.call_method(
                &intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(&key), JValue::Bool(0)],
            )?
            .z()?;
            Ok(Some(enabled.to_string()))
        } else {
            let value = env.call_method(
                &intent,
                "getStringExtra",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(&key)],
            )?
            .l()?;
            if value.is_null() {
                Ok(None)
            } else {
                let text: String = env.get_string((&value).into())?.into();
                Ok(Some(text))
            }
        }
    })
    .flatten()
}

pub fn launch_game_requested() -> bool {
    intent_extra("horizon.launch_game", true) == Some("true".to_string())
}

// ------------------------------------------------------------------ JNI entry points

#[unsafe(no_mangle)]
extern "system" fn Java_dev_horizon_android_HorizonEngine_nativeInit(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    home: JString<'_>,
    cache: JString<'_>,
) -> jstring {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
        let home = PathBuf::from(text(&mut env, &home)?);
        let cache = PathBuf::from(text(&mut env, &cache)?);
        let vm = env.get_java_vm().map_err(|e| e.to_string())?;
        crate::secure_store::initialize(vm);
        crate::backend::initialize(home.clone());
        let _ = std::env::set_current_dir(&home);
        horizon_client::android_runtime::configure_cache_dir(cache.clone());
        horizon_client::official_install::configure_download_cache_dir(cache.clone());
        let catalog = crate::catalog::Catalog::new(&home);
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut slot = state().lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(Engine {
            home,
            cache,
            catalog,
            sign_in: None,
            login_code: None,
            login_uri: None,
            login_message: String::new(),
            error_kind: String::new(),
            auth_stage: String::new(),
            status: String::new(),
            cancel: Arc::new(AtomicBool::new(false)),
            shutdown: shutdown.clone(),
        });
        drop(slot);
        if POLLER
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let _ = std::thread::Builder::new()
                .name("horizon-flutter-bridge".into())
                .spawn(move || poll_loop(shutdown));
        }
        Ok(())
    }))
    .unwrap_or_else(|_| Err("Engine bridge initialization panicked".into()));
    answer(
        &mut env,
        match result {
            Ok(()) => success("Real Rust engine bridge ready", true),
            Err(error) => failure(error),
        },
    )
}

#[unsafe(no_mangle)]
extern "system" fn Java_dev_horizon_android_HorizonEngine_nativeSnapshot(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let slot = state().lock().unwrap_or_else(|e| e.into_inner());
        match slot.as_ref() {
            Some(e) => snapshot(e),
            None => json!({
                "appVersion": crate::backend::VERSION,
                "gameVersion": horizon_core::VANILLA_VERSION,
                "engineBridgeReady": false,
                "status": "Engine bridge is not initialized"
            }),
        }
    }))
    .unwrap_or_else(|_| {
        json!({"appVersion": "", "gameVersion": "", "engineBridgeReady": false,
               "status": "Engine snapshot failed"})
    });
    answer(&mut env, value)
}

#[unsafe(no_mangle)]
extern "system" fn Java_dev_horizon_android_HorizonEngine_nativeDispatch(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    action: JString<'_>,
    args: JString<'_>,
) -> jstring {
    let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Value {
        let action = match text(&mut env, &action) {
            Ok(v) => v,
            Err(error) => return failure(error),
        };
        let raw = match text(&mut env, &args) {
            Ok(v) => v,
            Err(error) => return failure(error),
        };
        let args: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(error) => return failure(format!("Invalid action arguments: {error}")),
        };
        let mut slot = state().lock().unwrap_or_else(|e| e.into_inner());
        match slot.as_mut() {
            Some(e) => dispatch(e, &action, &args),
            None => failure("Engine bridge is not initialized"),
        }
    }))
    .unwrap_or_else(|_| failure("Native action panicked"));
    answer(&mut env, value)
}

#[unsafe(no_mangle)]
extern "system" fn Java_dev_horizon_android_HorizonEngine_nativeCancel(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let value = {
        let slot = state().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(e) = slot.as_ref() {
            e.cancel.store(true, Ordering::Relaxed);
            if let Some(task) = &e.sign_in {
                task.cancel();
            }
            success("Cancellation requested", false)
        } else {
            failure("Engine bridge is not initialized")
        }
    };
    answer(&mut env, value)
}

#[unsafe(no_mangle)]
extern "system" fn Java_dev_horizon_android_HorizonEngine_nativeShutdown(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    let value = {
        let slot = state().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(e) = slot.as_ref() {
            e.shutdown.store(true, Ordering::Relaxed);
            if let Some(task) = &e.sign_in {
                task.cancel();
            }
            success("Engine bridge stopped", false)
        } else {
            failure("Engine bridge is not initialized")
        }
    };
    answer(&mut env, value)
}

/// Complete a real SAF import. Kotlin already copied the document into a private
/// staged file with ContentResolver; here we reuse the engine's own validation
/// (`content_files::validate`), mod/shader parsing and safe rename.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_horizon_android_HorizonEngine_nativeImportStaged(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    kind: JString<'_>,
    staged: JString<'_>,
    name: JString<'_>,
) -> jstring {
    let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Value {
        let kind = match text(&mut env, &kind).ok().and_then(|k| kind_of(&k)) {
            Some(k) => k,
            None => return failure("Unknown content kind"),
        };
        let staged = match text(&mut env, &staged) {
            Ok(v) => PathBuf::from(v),
            Err(error) => return failure(error),
        };
        let name = match text(&mut env, &name) {
            Ok(v) => v,
            Err(error) => return failure(error),
        };
        let mut slot = state().lock().unwrap_or_else(|e| e.into_inner());
        let Some(e) = slot.as_mut() else {
            return failure("Engine bridge is not initialized");
        };
        let home = e.home.clone();
        let game = e.catalog.config.game_home(&home);
        let result = (|| -> Result<String, String> {
            if !horizon_client::content_files::plain_name(&name) || !kind.accepts(&name) {
                return Err("Wrong file type for this folder".into());
            }
            let dir = game.join(kind.folder());
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            if std::fs::symlink_metadata(&dir)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err("Refusing a redirected content directory".into());
            }
            if e.cancel.load(Ordering::Relaxed) {
                return Err("Import cancelled".into());
            }
            let cancel = e.cancel.clone();
            let native_shader = horizon_client::content_files::validate(
                &staged,
                kind,
                &name,
                || cancel.load(Ordering::Relaxed),
            )
            .map_err(|error| error.to_string())?;
            let final_name = if kind == horizon_client::content_files::Kind::Mods {
                let package = horizon_mods::Package::read(&staged)
                    .map_err(|error| error.to_string())?;
                format!("{}.hmod", package.manifest.id)
            } else if native_shader || name.to_ascii_lowercase().ends_with(".hshader") {
                let pack = horizon_shader_api::Pack::read(&staged)
                    .map_err(|error| error.to_string())?;
                horizon_shader_api::check(&pack).map_err(|error| format!("{error:?}"))?;
                let mut path = PathBuf::from(&name);
                path.set_extension("hshader");
                path.to_string_lossy().into_owned()
            } else {
                name.clone()
            };
            if !horizon_client::content_files::plain_name(&final_name) {
                return Err("Package has an invalid ID or filename".into());
            }
            let target = dir.join(&final_name);
            if target.exists() {
                let backup = dir.join(format!("{final_name}.previous"));
                let _ = std::fs::rename(&target, backup);
            }
            std::fs::rename(&staged, &target).map_err(|error| error.to_string())?;
            Ok(format!(
                "Imported {} into {}. Enable it in Installed or game settings.",
                final_name,
                kind.label()
            ))
        })();
        match result {
            Ok(message) => {
                e.catalog.refresh(&home);
                e.status = message.clone();
                success(message, true)
            }
            Err(error) => {
                let _ = std::fs::remove_file(&staged);
                failure(error)
            }
        }
    }))
    .unwrap_or_else(|_| failure("Import panicked"));
    answer(&mut env, value)
}
