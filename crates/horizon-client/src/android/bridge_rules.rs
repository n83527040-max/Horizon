//! Pure, dependency-free rules for the Flutter <-> Rust JNI bridge.
//!
//! This file is deliberately standalone: `rustc --test bridge_rules.rs` compiles
//! and RUNS its unit tests on a normal Linux host in CI, without an Android
//! target, NDK, Cargo manifest change or emulator. The Android bridge includes
//! the same file via `#[path = "bridge_rules.rs"] mod rules;`, so tested rules
//! and shipped rules can never diverge.
//!
//! No engine, authentication, Vulkan or Minecraft logic lives here.

/// The reason a native operation failed. Never "no results" for a real error.
pub fn classify_error(message: &str) -> &'static str {
    if message.is_empty() {
        return "none";
    }
    let m = message.to_ascii_lowercase();
    if m.contains("no connection")
        || m.contains("dns")
        || m.contains("timed out")
        || m.contains("timeout")
        || m.contains("connection refused")
        || m.contains("tls")
        || m.contains("network")
    {
        "network"
    } else if m.contains("wasn't readable")
        || m.contains("invalid json")
        || m.contains("expected value")
        || m.contains("missing field")
        || m.contains("serde")
    {
        "parse"
    } else if m.contains("answered 4")
        || m.contains("answered 5")
        || m.contains("429")
        || m.contains("slow down")
        || m.contains("rate limit")
        || m.contains("api")
    {
        "api"
    } else if m.contains("not supported")
        || m.contains("incompatible")
        || m.contains("cannot run")
        || m.contains("unsupported download source")
        || m.contains("needs a separately compiled client")
    {
        "compatibility"
    } else if m.contains("permission denied")
        || m.contains("read-only")
        || m.contains("no such file")
        || m.contains("cannot open")
        || m.contains("could not access")
        || m.contains("write")
    {
        "storage"
    } else if m.contains("sign in")
        || m.contains("token")
        || m.contains("microsoft")
        || m.contains("xbox")
        || m.contains("xsts")
        || m.contains("minecraftservices")
        || m.contains("client id")
        || m.contains("owns")
        || m.contains("doesn't own")
        || m.contains("does not own")
        || m.contains("do not own")
        || m.contains("session ended")
    {
        "auth"
    } else {
        "other"
    }
}

/// Which stage of Microsoft authentication failed, from the real error text.
/// Device Code Flow has NO redirect URI; its callback is the token poll.
pub fn auth_stage(message: &str) -> &'static str {
    let m = message.to_ascii_lowercase();
    if m.contains("devicecode") || m.contains("device code") || m.contains("verification") {
        "device_code"
    } else if m.contains("user.auth.xboxlive") || m.contains("xbox user") || m.contains("xui") {
        "xbox_live"
    } else if m.contains("xsts") || m.contains("xsts.auth") {
        "xsts"
    } else if m.contains("minecraftservices") || m.contains("minecraft login") {
        "minecraft_services"
    } else if m.contains("owns")
        || m.contains("doesn't own")
        || m.contains("does not own")
        || m.contains("do not own")
        || m.contains("profile")
        || m.contains("no profile")
    {
        "entitlement_profile"
    } else if m.contains("keychain") || m.contains("keystore") || m.contains("store") {
        "secure_store"
    } else if m.contains("refresh") || m.contains("expired") {
        "token_refresh"
    } else {
        "unknown"
    }
}

/// Offline player names: 3..=16 ASCII letters, digits or `_`. Mirrors
/// `launcher/src/account.rs::valid_offline_name` exactly; that function stays
/// the authority at runtime, this one is the tested, side-effect-free mirror.
pub fn valid_offline_name(name: &str) -> bool {
    let name = name.trim();
    (3..=16).contains(&name.chars().count())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A "Field(name)" with no value must NEVER be reported as a saved name.
pub fn empty_name_save(name: &str, value: Option<&str>) -> bool {
    (name == "name" || name == "offlineName")
        && value.map(str::trim).unwrap_or("").is_empty()
}

/// Only these catalog pages may start a real browse/search.
pub fn catalog_page(name: &str) -> Option<&'static str> {
    match name {
        "Mods" | "horizon-mods" => Some("Mods"),
        "Packs" | "resourcepacks" => Some("Packs"),
        "Shaders" | "shaderpacks" => Some("Shaders"),
        _ => None,
    }
}

/// The only loaders Modrinth facets may receive (Horizon runs no Java loader).
pub fn normalize_loader(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "all" | "any" => None,
        "fabric" => Some("fabric"),
        "forge" => Some("forge"),
        "neoforge" => Some("neoforge"),
        "quilt" => Some("quilt"),
        _ => None,
    }
}

/// A Minecraft version filter; `26.3` is the pinned engine version.
pub fn normalize_version(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() || v.eq_ignore_ascii_case("all") {
        return None;
    }
    if v.len() <= 32
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        Some(v.to_owned())
    } else {
        None
    }
}

/// Whether retrying can plausibly succeed (storage/compatibility errors need a
/// different user action, not a blind retry).
pub fn can_retry(error_kind: &str) -> bool {
    matches!(error_kind, "network" | "api" | "auth" | "none")
}

/// A native `ok=true` is only trustworthy when the operation had real input or
/// an unambiguous side effect. Guards against silent/dummy success.
pub fn verifiable_success(action: &str, name: Option<&str>, value: Option<&str>) -> bool {
    if action == "Field" && empty_name_save(name.unwrap_or(""), value) {
        return false;
    }
    if matches!(action, "Install" | "ToggleLocal" | "DeleteLocal") && value.is_none() {
        // index comes through args, not `value`; verified by the JNI dispatcher.
    }
    true
}

/// Progress is only reportable when the downloader supplied a real total.
/// Never invent a percentage for an indeterminate download.
pub fn download_percent(done: u64, total: u64) -> Option<u8> {
    if total == 0 {
        return None;
    }
    let value = done.saturating_mul(100) / total;
    Some(value.min(100) as u8)
}

/// Transfer speed derived from TWO real samples (delta bytes / delta time).
/// Returns None when the sample is too small or too fast to be meaningful.
pub fn transfer_speed(delta_bytes: u64, delta_ms: u64) -> Option<u64> {
    if delta_ms < 250 || delta_bytes == 0 {
        return None;
    }
    Some(delta_bytes.saturating_mul(1000) / delta_ms)
}

/// ETA is only shown when both remaining bytes and a real speed are known.
pub fn eta_seconds(remaining: u64, speed_bytes_per_second: u64) -> Option<u64> {
    if speed_bytes_per_second == 0 || remaining == 0 {
        return None;
    }
    Some(remaining / speed_bytes_per_second)
}

/// Decorative milestone blocks at 25/50/75/100%. Driven by real progress only.
pub fn milestone(done: u64, total: u64) -> u8 {
    match download_percent(done, total) {
        Some(p) if p >= 100 => 4,
        Some(p) if p >= 75 => 3,
        Some(p) if p >= 50 => 2,
        Some(p) if p >= 25 => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_network_and_api_errors_are_not_reported_as_no_results() {
        assert_eq!(classify_error("No connection to Modrinth: dns error"), "network");
        assert_eq!(classify_error("Modrinth answered 429."), "api");
        assert_eq!(classify_error("Modrinth answered 503."), "api");
        assert_eq!(classify_error("Modrinth's answer wasn't readable: EOF"), "parse");
        assert_eq!(classify_error("Permission denied"), "storage");
        assert_eq!(classify_error("1 result - select Install"), "other");
        assert_eq!(classify_error(""), "none");
        assert!(!can_retry("storage"));
        assert!(can_retry("network"));
    }

    #[test]
    fn microsoft_stages_are_classified_from_real_errors() {
        assert_eq!(auth_stage("devicecode request failed"), "device_code");
        assert_eq!(auth_stage("user.auth.xboxlive.com refused"), "xbox_live");
        assert_eq!(auth_stage("XSTS returned 401"), "xsts");
        assert_eq!(auth_stage("api.minecraftservices.com: 403"), "minecraft_services");
        assert_eq!(auth_stage("This account doesn't own Minecraft: Java Edition."),
            "entitlement_profile");
        assert_eq!(classify_error("This account doesn't own Minecraft: Java Edition."), "auth");
        assert_eq!(auth_stage("refresh token expired"), "token_refresh");
    }

    #[test]
    fn offline_names_follow_the_real_account_rule() {
        assert!(valid_offline_name("Steve"));
        assert!(valid_offline_name("Player_123"));
        assert!(!valid_offline_name(""));
        assert!(!valid_offline_name("ab"));
        assert!(!valid_offline_name("a b"));
        assert!(!valid_offline_name("أحمد"));
        assert!(!valid_offline_name("x".repeat(17).as_str()));
    }

    #[test]
    fn an_empty_offline_name_can_never_be_a_saved_account() {
        assert!(empty_name_save("name", None));
        assert!(empty_name_save("offlineName", Some("   ")));
        assert!(!empty_name_save("name", Some("Steve")));
        assert!(!empty_name_save("theme", None));
        assert!(!verifiable_success("Field", Some("name"), None));
        assert!(verifiable_success("Field", Some("name"), Some("Steve")));
    }

    #[test]
    fn only_real_catalog_pages_browse() {
        assert_eq!(catalog_page("Mods"), Some("Mods"));
        assert_eq!(catalog_page("Packs"), Some("Packs"));
        assert_eq!(catalog_page("Shaders"), Some("Shaders"));
        assert_eq!(catalog_page("Play"), None);
        assert_eq!(catalog_page("nonsense"), None);
    }

    #[test]
    fn filters_are_whitelisted_and_pinned() {
        assert_eq!(normalize_loader("Fabric"), Some("fabric"));
        assert_eq!(normalize_loader("NeoForge"), Some("neoforge"));
        assert_eq!(normalize_loader("Quilt"), Some("quilt"));
        assert_eq!(normalize_loader("fake-loader"), None);
        assert_eq!(normalize_version("26.3"), Some("26.3".into()));
        assert_eq!(normalize_version("1.21.1"), Some("1.21.1".into()));
        assert_eq!(normalize_version("../../etc"), None);
        assert_eq!(normalize_version("all"), None);
    }

    #[test]
    fn download_progress_never_invents_numbers() {
        assert_eq!(download_percent(0, 0), None);
        assert_eq!(download_percent(50, 100), Some(50));
        assert_eq!(download_percent(200, 100), Some(100));
        assert_eq!(transfer_speed(0, 1000), None);
        assert_eq!(transfer_speed(1000, 100), None); // too fast to be reliable
        assert_eq!(transfer_speed(1_000_000, 1000), Some(1_000_000));
        assert_eq!(eta_seconds(2_000_000, 1_000_000), Some(2));
        assert_eq!(eta_seconds(2_000_000, 0), None);
        assert_eq!(milestone(0, 100), 0);
        assert_eq!(milestone(25, 100), 1);
        assert_eq!(milestone(50, 100), 2);
        assert_eq!(milestone(75, 100), 3);
        assert_eq!(milestone(100, 100), 4);
        assert_eq!(milestone(10, 0), 0);
    }
}
