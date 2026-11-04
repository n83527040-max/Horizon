use super::*;
use horizon_client::osk::Output;
use winit::{
    event::{KeyEvent, Touch},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Action {
    Page(Page),
    Play,
    Search,
    ShaderSource(crate::shader_catalog::Origin),
    ModSource(crate::shader_catalog::Origin),
    Field(&'static str),
    FinishNameEdit,
    Install(usize),
    Import(horizon_client::content_files::Kind),
    OpenFolder(horizon_client::content_files::Kind),
    ToggleSection(usize),
    ToggleLocal(usize),
    DeleteLocal(usize),
    ConfirmDeleteLocal,
    Next,
    Previous,
    Profile,
    Logs,
    RecordPerformance,
    ExportPerformance,
    ExportDiagnostics,
    Account,
    SignIn,
    DeviceSignIn,
    CopyLoginCode,
    CancelLogin,
    ReviewApp,
    Repository,
    SignOut,
    LoginLink,
    CloseLogin,
    Renderer,
    RenderScale,
    ChooseRenderScale(crate::display::RenderScale),
    Orientation,
    InputHost,
    ChooseInputHost(crate::catalog::InputHost),
    ChooseOrientation(crate::catalog::Orientation),
    ChooseRenderer(crate::catalog::RendererMode),
    ChooseProfile(u32),
    Dismiss,
    Immersive,
    Cutout,
    Aspect,
    ChooseAspect(crate::display::Aspect),
    DisplayHz,
    ChooseDisplayHz(u16),
    PerformanceHints,
    GpuProfile,
}

use super::design;
use crate::browser::{Project, SourceId};
use crate::catalog::{Catalog, Page};
use crate::shader_catalog::Origin;
use horizon_client::content_files::Kind;

type Targets = Vec<([f32; 4], Action)>;

/// The shared button used by the launcher, the account panel and the value
/// chooser. Same signature and same hit target as the original: only its look
/// changes, so dialogs keep working without touching their logic.
pub(super) fn button(
    ui: &mut Ui,
    targets: &mut Targets,
    rect: [f32; 4],
    label: &str,
    action: Action,
    accent: bool,
) {
    let [x, y, w, h] = rect;
    let t = design::Theme::new(1.0);
    if accent {
        design::gradient_button(ui, &t, rect, 0.0);
    } else {
        design::card_alt(ui, &t, rect, design::radius("chip"));
    }
    let font = if w < 80.0 { 12.0 } else { 14.0 };
    let fitted = ui.fit_text(Face::Medium, font, label, w - 14.0);
    let width = ui.measure(Face::Medium, font, &fitted);
    ui.text(
        Face::Medium,
        font,
        x + (w - width) * 0.5,
        y + (h - font * 1.45) * 0.5,
        t.text(),
        &fitted,
    );
    targets.push((rect, action));
}
fn toggle(
    ui: &mut Ui,
    targets: &mut Targets,
    rect: [f32; 4],
    on: bool,
    action: Action,
) {
    let [x, y, w, h] = rect;
    let t = design::Theme::new(1.0);
    ui.text(
        Face::Regular,
        13.0,
        x,
        y + (h - 19.0) * 0.5,
        t.muted(),
        if on { "On" } else { "Off" },
    );
    let track_w = 46.0;
    let track = [x + w - track_w - 6.0, y + (h - 26.0) * 0.5, track_w, 26.0];
    ui.rounded(
        track[0], track[1], track[2], track[3], 13.0,
        if on { t.primary() } else { t.card_alt() },
    );
    ui.border(track[0], track[1], track[2], track[3], 13.0, t.outline());
    ui.rounded(
        track[0] + if on { 24.0 } else { 3.0 },
        track[1] + 3.0,
        20.0,
        20.0,
        10.0,
        t.text(),
    );
    targets.push((rect, action));
}
fn centered(ui: &mut Ui, t: &design::Theme, r: [f32; 4], size: f32, text: &str, face: Face) {
    let fitted = ui.fit_text(face, size, text, r[2] - 10.0);
    let width = ui.measure(face, size, &fitted);
    ui.text(
        face,
        size,
        r[0] + (r[2] - width) * 0.5,
        r[1] + (r[3] - size * 1.45) * 0.5,
        t.text(),
        &fitted,
    );
}
fn label_card(
    ui: &mut Ui,
    t: &design::Theme,
    r: [f32; 4],
    icon: design::Icon,
    label: &str,
    accent: &str,
    action: Option<Action>,
    targets: &mut Targets,
) {
    design::card_alt(ui, t, r, design::radius("chip"));
    design::icon(ui, icon, r[0] + 12.0, r[1] + (r[3] - 22.0) * 0.5, 22.0, t.tint(accent, 1.0));
    let fitted = ui.fit_text(Face::Medium, 13.0, label, r[2] - 48.0);
    ui.text(
        Face::Medium,
        13.0,
        r[0] + 42.0,
        r[1] + (r[3] - 19.0) * 0.5,
        t.text(),
        &fitted,
    );
    if let Some(action) = action {
        targets.push((r, action));
    }
}
fn draw_glows(ui: &mut Ui, t: &design::Theme, w: f32, h: f32) {
    // Two large, very low-alpha plates. No blur passes and no extra textures.
    ui.rounded(w * 0.52, -80.0, w * 0.78, 250.0, 125.0, design::alpha(t.primary(), 0.055));
    ui.rounded(-100.0, h * 0.60, w * 0.75, 280.0, 140.0, design::alpha(t.secondary(), 0.05));
}
fn draw_header(ui: &mut Ui, t: &design::Theme, l: &design::Layout, targets: &mut Targets, version: &str) {
    let (x, y, w) = (l.content_x, l.header_y, l.content_w);
    // The Horizon mark: two stacked triangular plates in the identity gradient.
    for i in 0..6 {
        let width = 34.0 - i as f32 * 5.0;
        ui.rounded(
            x + (34.0 - width) * 0.5,
            y + 7.0 + i as f32 * 5.0,
            width,
            4.0,
            2.0,
            if i < 3 { t.primary() } else { t.secondary() },
        );
    }
    ui.rounded(x + 8.0, y + 35.0, 18.0, 3.0, 1.5, t.cyan());
    ui.text(Face::SemiBold, 22.0, x + 46.0, y + 2.0, t.text(), "HORIZON");
    ui.text(Face::Regular, 11.0, x + 46.0, y + 28.0, t.muted(), "Minecraft Java • Android");
    let gear = [x + w - 40.0, y + 8.0, 40.0, 40.0];
    design::card_alt(ui, t, gear, design::radius("chip"));
    design::icon(ui, design::Icon::Gear, gear[0] + 9.0, gear[1] + 9.0, 22.0, t.text());
    targets.push((gear, Action::Page(Page::Settings)));
    let badge = [gear[0] - 86.0, y + 14.0, 78.0, 28.0];
    design::badge(ui, t, badge, &format!("v{version}"));
}
fn draw_subheader(ui: &mut Ui, t: &design::Theme, l: &design::Layout, title: &str, subtitle: &str) {
    let x = l.content_x;
    let y = l.page_top;
    let title = ui.fit_text(Face::SemiBold, 22.0, title, l.content_w);
    let subtitle = ui.fit_text(Face::Regular, 12.0, subtitle, l.content_w);
    ui.text(Face::SemiBold, 22.0, x, y + 1.0, t.text(), &title);
    ui.text(Face::Regular, 12.0, x, y + 29.0, t.muted(), &subtitle);
}
fn small_card(
    ui: &mut Ui,
    t: &design::Theme,
    r: [f32; 4],
    icon: design::Icon,
    accent: &str,
    title: &str,
    value: &str,
    detail: &str,
    action: Option<Action>,
    targets: &mut Targets,
) {
    design::glass_card(ui, t, r, design::radius("card"));
    design::icon(ui, icon, r[0] + 14.0, r[1] + 12.0, 24.0, t.tint(accent, 1.0));
    ui.text(Face::Regular, 11.0, r[0] + 46.0, r[1] + 11.0, t.muted(), title);
    let value = ui.fit_text(Face::SemiBold, 17.0, value, r[2] - 60.0);
    ui.text(Face::SemiBold, 17.0, r[0] + 14.0, r[1] + 31.0, t.text(), &value);
    let detail = ui.fit_text(Face::Regular, 11.0, detail, r[2] - 60.0);
    ui.text(Face::Regular, 11.0, r[0] + 14.0, r[1] + 55.0, t.muted(), &detail);
    if let Some(action) = action {
        design::icon(ui, design::Icon::Chevron, r[0] + r[2] - 30.0, r[1] + 28.0, 20.0, t.dim());
        targets.push((r, action));
    }
}
fn feature_card(
    ui: &mut Ui,
    t: &design::Theme,
    r: [f32; 4],
    icon: design::Icon,
    accent: &str,
    title: &str,
    detail: &str,
    action: Action,
    targets: &mut Targets,
) {
    design::card(ui, t, r, design::radius("card"));
    design::icon(ui, icon, r[0] + 14.0, r[1] + 13.0, 26.0, t.tint(accent, 1.0));
    ui.rounded(r[0] + r[2] - 30.0, r[1] + 14.0, 8.0, 8.0, 4.0, design::alpha(t.tint(accent, 1.0), 0.7));
    ui.text(Face::SemiBold, 15.0, r[0] + 14.0, r[1] + 44.0, t.text(), title);
    let detail = ui.fit_text(Face::Regular, 11.0, detail, r[2] - 44.0);
    ui.text(Face::Regular, 11.0, r[0] + 14.0, r[1] + 63.0, t.muted(), &detail);
    design::icon(ui, design::Icon::Chevron, r[0] + r[2] - 30.0, r[1] + 56.0, 18.0, t.dim());
    targets.push((r, action));
}

struct HomeData {
    version: String,
    mc_version: String,
    profile_name: String,
    account_status: String,
    signed_in: bool,
    mods: usize,
    packs: usize,
    shaders_enabled: bool,
    shader_packs: usize,
    installed: usize,
    installing: bool,
    install_label: String,
    install_detail: String,
    install_fraction: f32,
    renderer: String,
    resolution: u32,
    memory: design::Memory,
}

fn draw_home(
    ui: &mut Ui,
    t: &design::Theme,
    l: &design::Layout,
    targets: &mut Targets,
    scroll: f32,
    d: &HomeData,
    has_hero: bool,
) {
    let home = l.home(scroll);
    let (top, bottom) = (l.page_top, l.page_bottom);
    // ---- Hero banner ----
    if design::visible(home.hero, top, bottom) {
        design::glass_card(ui, t, home.hero, design::radius("hero"));
        let inner = [home.hero[0] + 3.0, home.hero[1] + 3.0, home.hero[2] - 6.0, home.hero[3] - 6.0];
        if has_hero {
            ui.image(horizon_client::ui_theme::HERO_SPRITE, inner[0], inner[1], inner[2], inner[3], t.text());
        } else {
            ui.gradient(inner[0], inner[1], inner[2], inner[3], t.primary(), t.secondary());
            for i in 0..7 {
                let bw = inner[2] * (0.10 + i as f32 * 0.035);
                ui.rounded(inner[0] + i as f32 * 38.0, inner[1] + inner[3] - 30.0 - i as f32 * 7.0, bw, 34.0, 3.0, design::alpha(t.card(), 0.55));
            }
        }
        // Legibility veil over the lower third only; the artwork stays visible.
        ui.gradient(
            inner[0], inner[1] + inner[3] * 0.44, inner[2], inner[3] * 0.56,
            design::alpha(t.bg(), 0.0), design::alpha(t.bg(), 0.86),
        );
        for i in 0..4 {
            ui.rounded(inner[0] + 18.0 + i as f32 * 3.0, inner[1] + inner[3] - 94.0 + i as f32 * 5.0, 20.0 - i as f32 * 5.0, 4.0, 2.0, t.primary());
        }
        ui.text(Face::SemiBold, 26.0, inner[0] + 18.0, inner[1] + inner[3] - 70.0, t.text(), "HORIZON");
        let edition = ui.fit_text(Face::Regular, 13.0, "Minecraft Java Edition", inner[2] - 36.0);
        ui.text(Face::Regular, 13.0, inner[0] + 18.0, inner[1] + inner[3] - 38.0, t.text(), &edition);
    }
    // ---- Play ----
    if design::visible(home.play, top, bottom) {
        design::gradient_button(ui, t, home.play, if d.installing { 0.8 } else { 0.0 });
        let label = ui.fit_text(Face::SemiBold, 20.0, &d.install_label, home.play[2] - 80.0);
        let width = ui.measure(Face::SemiBold, 20.0, &label);
        let start = home.play[0] + (home.play[2] - width - 34.0) * 0.5;
        design::icon(ui, design::Icon::Play, start, home.play[1] + (home.play[3] - 24.0) * 0.5, 24.0, t.text());
        ui.text(Face::SemiBold, 20.0, start + 32.0, home.play[1] + (home.play[3] - 29.0) * 0.5, t.text(), &label);
        if d.installing {
            design::progress(ui, t, [home.play[0] + 16.0, home.play[1] + home.play[3] - 9.0, home.play[2] - 32.0, 4.0], d.install_fraction, "cyan");
        }
        targets.push((home.play, Action::Play));
    }
    // ---- Version + profile ----
    if design::visible(home.version, top, bottom) {
        small_card(ui, t, home.version, design::Icon::Cube, "cyan", "Minecraft", &d.mc_version, "Pinned engine version", None, targets);
    }
    if design::visible(home.profile, top, bottom) {
        small_card(
            ui, t, home.profile, design::Icon::User, "primary", "Profile", &d.profile_name,
            &d.account_status, Some(Action::Account), targets,
        );
    }
    // ---- Manage ----
    if design::visible(home.manage_label, top, bottom) {
        design::section_title(ui, t, home.manage_label, "Manage");
        ui.text(Face::Regular, 11.0, home.manage_label[0] + 74.0, home.manage_label[1] + 3.0, t.dim(), "Content on this device");
    }
    let feature_data = [
        (design::Icon::Puzzle, "primary", "Mods", format!("{} installed", d.mods), Action::Page(Page::Mods)),
        (design::Icon::Layers, "cyan", "Packs", format!("{} installed", d.packs), Action::Page(Page::Packs)),
        (design::Icon::Sparkle, "secondary", "Shaders", if d.shaders_enabled { "Enabled".into() } else { format!("{} available", d.shader_packs) }, Action::Page(Page::Shaders)),
        (design::Icon::Download, "primary", "Installed", format!("{} items", d.installed), Action::Page(Page::Installed)),
    ];
    for (i, (ic, accent, title, detail, action)) in feature_data.iter().enumerate() {
        let r = home.features[i];
        if design::visible(r, top, bottom) {
            feature_card(ui, t, r, *ic, accent, title, detail.as_str(), *action, targets);
        }
    }
    // ---- Performance: real /proc and real config only ----
    if design::visible(home.performance, top, bottom) {
        let r = home.performance;
        design::glass_card(ui, t, r, design::radius("card"));
        design::icon(ui, design::Icon::Gauge, r[0] + 14.0, r[1] + 12.0, 22.0, t.cyan());
        design::section_title(ui, t, [r[0] + 44.0, r[1] + 10.0, r[2] - 90.0, 22.0], "Performance");
        design::icon(ui, design::Icon::Chevron, r[0] + r[2] - 30.0, r[1] + 12.0, 20.0, t.dim());
        let mut line = r[1] + 38.0;
        match (d.memory.rss_mb, d.memory.device_mb) {
            (Some(rss), Some(device)) => {
                let fraction = rss as f32 / device.max(1) as f32;
                ui.text(Face::Regular, 11.0, r[0] + 14.0, line, t.muted(), "RAM usage");
                let value = format!("{} / {}", design::gib(rss), design::gib(device));
                let value = ui.fit_text(Face::Medium, 11.0, &value, r[2] - 130.0);
                let width = ui.measure(Face::Medium, 11.0, &value);
                ui.text(Face::Medium, 11.0, r[0] + r[2] - 14.0 - width, line, t.text(), &value);
                design::progress(ui, t, [r[0] + 14.0, line + 18.0, r[2] - 28.0, 6.0], fraction, "primary");
                line += 30.0;
            }
            _ => {
                ui.text(Face::Regular, 11.0, r[0] + 14.0, line, t.dim(), "Memory data unavailable on this device");
                line += 18.0;
            }
        }
        let api = ui.fit_text(Face::Regular, 11.0, &d.renderer, r[2] * 0.52);
        ui.text(Face::Regular, 11.0, r[0] + 14.0, line, t.muted(), "Graphics API");
        ui.text(Face::Regular, 11.0, r[0] + 14.0, line + 15.0, t.text(), &api);
        ui.text(Face::Regular, 11.0, r[0] + r[2] * 0.58, line, t.muted(), "Resolution");
        let resolution = format!("{}%", d.resolution);
        ui.text(Face::Regular, 11.0, r[0] + r[2] * 0.58, line + 15.0, t.text(), &resolution);
        if let Some(peak) = d.memory.peak_mb {
            let peak = format!("Peak {}", design::gib(peak));
            ui.text(Face::Regular, 10.0, r[0] + 14.0, r[1] + r[3] - 18.0, t.dim(), &peak);
        }
        targets.push((r, Action::Page(Page::Settings)));
    }
    // ---- Footer / source ----
    if design::visible(home.toast, top, bottom) {
        let r = home.toast;
        design::card(ui, t, r, design::radius("chip"));
        let text = format!("v{}  •  Minecraft Java {}", d.version, d.mc_version);
        let text = ui.fit_text(Face::Regular, 11.0, &text, r[2] - 110.0);
        ui.text(Face::Regular, 11.0, r[0] + 14.0, r[1] + 13.0, t.muted(), &text);
        let github = [r[0] + r[2] - 96.0, r[1] + 6.0, 88.0, 32.0];
        label_card(ui, t, github, design::Icon::Info, "GitHub", "cyan", Some(Action::Repository), targets);
        let _ = d.signed_in;
    }
}

fn draw_catalog_page(
    ui: &mut Ui,
    t: &design::Theme,
    l: &design::Layout,
    targets: &mut Targets,
    list_targets: &mut Targets,
    catalog: &mut Catalog,
    page: Page,
    phase: f32,
) -> Option<[f32; 3]> {
    let (x, w, gap) = (l.content_x, l.content_w, l.gap);
    let is_mods = page == Page::Mods;
    let is_shaders = page == Page::Shaders;
    let kind = if is_mods { Kind::Mods } else if is_shaders { Kind::Shaders } else { Kind::Packs };
    let title = if is_mods { "Mods" } else if is_shaders { "Shaders" } else { "Resource Packs" };
    let subtitle = if is_mods {
        "Native .hmod packages • install, enable or remove"
    } else if is_shaders {
        "Native .hshader packs • compatibility and activation"
    } else {
        "Verified downloads • enable in Installed"
    };
    draw_subheader(ui, t, l, title, subtitle);
    let mut y = l.content_top;
    // Three content destinations. Shaders is reachable here even though the
    // fixed bottom bar keeps exactly five items.
    let chip_h = design::metric("catalog", "chip_height");
    for (i, p) in [Page::Mods, Page::Packs, Page::Shaders].into_iter().enumerate() {
        let r = l.third(y, chip_h, i);
        design::chip(ui, t, r, p == page, "primary");
        centered(ui, t, r, 12.0, if p == Page::Packs { "Packs" } else if p == Page::Shaders { "Shaders" } else { "Mods" }, Face::Medium);
        targets.push((r, Action::Page(p)));
    }
    y += chip_h + gap;
    // Search / HTTPS .hmod import.
    let search_h = design::metric("catalog", "search_height");
    let search_w = (w - 96.0 - gap).max(120.0);
    let field = [x, y, search_w, search_h];
    design::card_alt(ui, t, field, design::radius("chip"));
    design::icon(ui, design::Icon::Search, x + 12.0, y + 12.0, 22.0, t.muted());
    let placeholder = if is_mods { "Search or paste an HTTPS .hmod link" } else { "Search packs" };
    let query = if catalog.query.is_empty() { placeholder.to_owned() } else { catalog.query.clone() };
    let query = ui.fit_text(Face::Regular, 13.0, &query, search_w - 46.0);
    ui.text(Face::Regular, 13.0, x + 42.0, y + 13.0, if catalog.query.is_empty() { t.dim() } else { t.text() }, &query);
    targets.push((field, Action::Field("search")));
    let search = [x + search_w + gap, y, 96.0, search_h];
    design::gradient_button(ui, t, search, 0.0);
    centered(ui, t, search, 13.0, if is_mods && catalog.query.trim().starts_with("https://") { "Import" } else { "Search" }, Face::Medium);
    targets.push((search, Action::Search));
    y += search_h + gap;
    // Real source switch (Horizon registry / Modrinth browsing).
    if is_mods || is_shaders {
        let source_w = (w - gap) * 0.5;
        for (i, origin) in [Origin::Horizon, Origin::Modrinth].into_iter().enumerate() {
            let r = [x + i as f32 * (source_w + gap), y, source_w, 32.0];
            let active = (if is_mods { catalog.config.mod_source } else { catalog.config.shader_source }) == origin;
            design::chip(ui, t, r, active, "cyan");
            centered(ui, t, r, 12.0, origin.source().label(), Face::Regular);
            targets.push((r, if is_mods { Action::ModSource(origin) } else { Action::ShaderSource(origin) }));
        }
        y += 32.0 + gap;
    }
    let action_h = design::metric("catalog", "action_height");
    let open = l.half(y, action_h, false);
    let import = l.half(y, action_h, true);
    label_card(ui, t, open, design::Icon::Folder, "Open folder", "cyan", Some(Action::OpenFolder(kind)), targets);
    label_card(ui, t, import, design::Icon::Download, "Import file", "primary", Some(Action::Import(kind)), targets);
    y += action_h + gap;
    let hint = if is_mods && catalog.config.mod_source == Origin::Modrinth {
        "Modrinth Java mods are browse-only; Fabric/Forge/NeoForge cannot run here"
    } else if is_mods {
        "Search native Horizon mods or paste an HTTPS .hmod link"
    } else if is_shaders && catalog.config.shader_source == Origin::Modrinth {
        "Modrinth shader ZIPs: experimental compatibility; some packs may not work"
    } else if is_shaders {
        "Horizon native .hshader packs"
    } else {
        "Verified downloads / enable in Installed"
    };
    let hint = ui.fit_text(Face::Regular, 11.0, hint, w);
    ui.text(Face::Regular, 11.0, x, y + 2.0, t.dim(), &hint);
    y += 20.0 + gap;
    let top = y;
    let bottom = l.page_bottom;
    let row_h = design::metric("catalog", "row_height");
    let pagination_h = 48.0;
    let row_count = catalog.results.len();
    let maximum = (row_count as f32 * row_h + pagination_h - (bottom - top)).max(0.0);
    catalog.scroll = catalog.scroll.clamp(0.0, maximum);
    let view = [top, bottom, maximum];
    if row_count == 0 && (catalog.searching() || catalog.busy()) {
        for i in 0..3 {
            let r = [x, top + i as f32 * (row_h + gap), w, row_h - 8.0];
            design::skeleton(ui, t, r, phase);
        }
    } else if row_count == 0 {
        let empty = [x, top + 8.0, w, 64.0];
        design::card(ui, t, empty, design::radius("card"));
        centered(ui, t, empty, 13.0, if catalog.query.is_empty() { "Search to find content" } else { "No results" }, Face::Regular);
    }
    let rows: Vec<(usize, &Project)> = catalog.results.iter().enumerate().collect();
    for (i, p) in rows {
        let r = [x, top + i as f32 * row_h - catalog.scroll, w, row_h - 8.0];
        let state = catalog.download_label(p);
        let source_only = crate::catalog::source_only_mod(p);
        let java_mod = p.kind == crate::browser::Kind::Mod && p.source == SourceId::Modrinth;
        let installable = state.is_none() && !source_only && !java_mod
            && (p.kind != crate::browser::Kind::Mod || cfg!(target_pointer_width = "64"));
        let install_rect = [r[0] + r[2] - 100.0, r[1] + 20.0, 88.0, 42.0];
        if installable {
            list_targets.push((install_rect, Action::Install(i)));
        }
        if !design::visible(r, top, bottom) {
            continue;
        }
        design::card(ui, t, r, design::radius("card"));
        // A deterministic letter tile: no fake downloaded artwork.
        let tile = [r[0] + 12.0, r[1] + 16.0, 48.0, 48.0];
        ui.rounded(tile[0], tile[1], tile[2], tile[3], 14.0, design::alpha(t.primary(), 0.22));
        let initial: String = p.title.chars().take(1).collect();
        centered(ui, t, tile, 20.0, &initial, Face::SemiBold);
        let title = ui.fit_text(Face::SemiBold, 15.0, &p.title, r[2] - 178.0);
        ui.text(Face::SemiBold, 15.0, r[0] + 70.0, r[1] + 12.0, t.text(), &title);
        let summary = ui.fit_text(Face::Regular, 11.0, &p.summary, r[2] - 178.0);
        ui.text(Face::Regular, 11.0, r[0] + 70.0, r[1] + 34.0, t.muted(), &summary);
        let version = p.game_versions.first().map(String::as_str).unwrap_or("26.3");
        let compatibility = format!("{}  •  {}", p.support().label(), version);
        let compatibility = ui.fit_text(Face::Regular, 10.0, &compatibility, r[2] - 178.0);
        ui.text(Face::Regular, 10.0, r[0] + 70.0, r[1] + 54.0, t.dim(), &compatibility);
        if let Some(state) = state {
            design::chip(ui, t, install_rect, false, "cyan");
            centered(ui, t, install_rect, 12.0, &state, Face::Medium);
        } else if source_only {
            centered(ui, t, install_rect, 11.0, ".hnmod: later", Face::Regular);
        } else if java_mod {
            centered(ui, t, install_rect, 11.0, "Java mod", Face::Regular);
        } else if p.kind == crate::browser::Kind::Mod && !cfg!(target_pointer_width = "64") {
            centered(ui, t, install_rect, 11.0, "64-bit", Face::Regular);
        } else {
            design::gradient_button(ui, t, install_rect, 0.0);
            centered(ui, t, install_rect, 12.0, catalog.install_button(p), Face::Medium);
            targets.push((install_rect, Action::Install(i)));
        }
    }
    let pagination_y = top + row_count as f32 * row_h - catalog.scroll + 4.0;
    let pagination = [x, pagination_y, w, 40.0];
    if design::visible(pagination, top, bottom) {
        let previous = l.half(pagination_y, 38.0, false);
        let next = l.half(pagination_y, 38.0, true);
        label_card(ui, t, previous, design::Icon::Chevron, "cyan", "Previous", Some(Action::Previous), targets);
        label_card(ui, t, next, design::Icon::Chevron, "cyan", "Next", Some(Action::Next), targets);
    }
    Some(view)
}

fn draw_installed_page(
    ui: &mut Ui,
    t: &design::Theme,
    l: &design::Layout,
    targets: &mut Targets,
    list_targets: &mut Targets,
    catalog: &mut Catalog,
) -> Option<[f32; 3]> {
    draw_subheader(ui, t, l, "Installed", "Enable, import or remove content by type");
    let (x, w, gap) = (l.content_x, l.content_w, l.gap);
    let top = l.content_top;
    let bottom = l.page_bottom;
    let section_h = design::metric("installed", "section_height");
    let row_h = design::metric("installed", "row_height");
    let mut total = 0.0;
    for (section, kind) in Kind::ALL.into_iter().enumerate() {
        total += section_h + gap;
        if catalog.sections[section] {
            total += catalog.local.iter().filter(|e| e.kind == kind).count() as f32 * row_h;
        }
    }
    let maximum = (total - (bottom - top)).max(0.0);
    catalog.scroll = catalog.scroll.clamp(0.0, maximum);
    let view = [top, bottom, maximum];
    let mut offset = 0.0;
    for (section, kind) in Kind::ALL.into_iter().enumerate() {
        let count = catalog.local.iter().filter(|e| e.kind == kind).count();
        let y = top + offset - catalog.scroll;
        let header = [x, y, w, section_h];
        let toggle_rect = [x, y, w - 110.0, section_h];
        list_targets.push((toggle_rect, Action::ToggleSection(section)));
        if design::visible(header, top, bottom) {
            design::card_alt(ui, t, header, design::radius("chip"));
            let icon = match kind {
                Kind::Mods => design::Icon::Puzzle,
                Kind::Packs => design::Icon::Layers,
                Kind::Shaders => design::Icon::Sparkle,
            };
            design::icon(ui, icon, x + 12.0, y + 11.0, 22.0, t.tint("primary", 1.0));
            let label = format!("{}  ({})", kind.label(), count);
            ui.text(Face::SemiBold, 14.0, x + 42.0, y + 12.0, t.text(), &label);
            design::icon(ui, design::Icon::Chevron, x + w - 136.0, y + 11.0, 20.0, t.dim());
            targets.push((toggle_rect, Action::ToggleSection(section)));
            let import = [x + w - 104.0, y + 6.0, 96.0, section_h - 12.0];
            label_card(ui, t, import, design::Icon::Download, "Import", "cyan", Some(Action::Import(kind)), targets);
        }
        offset += section_h + gap;
        if !catalog.sections[section] {
            continue;
        }
        let items: Vec<(usize, String, bool)> = catalog
            .local
            .iter()
            .enumerate()
            .filter(|(_, e)| e.kind == kind)
            .map(|(i, e)| (i, e.title.clone(), e.enabled))
            .collect();
        for (i, title, enabled) in items {
            let y = top + offset - catalog.scroll;
            let r = [x, y, w, row_h - 8.0];
            let switch = [x + 12.0, y + 46.0, 96.0, 30.0];
            let remove = [x + w - 104.0, y + 46.0, 92.0, 30.0];
            list_targets.push((switch, Action::ToggleLocal(i)));
            list_targets.push((remove, Action::DeleteLocal(i)));
            if !design::visible(r, top, bottom) {
                offset += row_h;
                continue;
            }
            design::card(ui, t, r, design::radius("card"));
            design::icon(ui, if kind == Kind::Mods { design::Icon::Puzzle } else { design::Icon::Layers }, x + 14.0, y + 12.0, 22.0, t.tint("cyan", 1.0));
            let title = ui.fit_text(Face::SemiBold, 14.0, &title, w - 150.0);
            ui.text(Face::SemiBold, 14.0, x + 44.0, y + 10.0, t.text(), &title);
            let status = if enabled { "Enabled" } else { "Disabled" };
            let status_color = if enabled { "success" } else { "muted" };
            ui.rounded(x + w - 108.0, y + 14.0, 92.0, 22.0, 11.0, design::alpha(t.tint(status_color, 1.0), 0.18));
            centered(ui, t, [x + w - 108.0, y + 14.0, 92.0, 22.0], 11.0, status, Face::Medium);
            if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                ui.text(Face::Regular, 11.0, x + 14.0, y + 52.0, t.dim(), "64-bit required");
            } else {
                design::card_alt(ui, t, switch, design::radius("chip"));
                centered(ui, t, switch, 11.0, if enabled { "Disable" } else { "Enable" }, Face::Medium);
                targets.push((switch, Action::ToggleLocal(i)));
            }
            design::card_alt(ui, t, remove, design::radius("chip"));
            centered(ui, t, remove, 11.0, "Remove", Face::Medium);
            targets.push((remove, Action::DeleteLocal(i)));
            offset += row_h;
        }
    }
    Some(view)
}

fn draw_settings_page(
    ui: &mut Ui,
    t: &design::Theme,
    l: &design::Layout,
    targets: &mut Targets,
    list_targets: &mut Targets,
    catalog: &mut Catalog,
) -> Option<[f32; 3]> {
    draw_subheader(ui, t, l, "Settings", "Graphics, performance, accounts and app");
    let c = &catalog.config;
    let scale_text = if catalog.editing == Some("render_scale") {
        catalog.render_scale_text.clone()
    } else {
        format!("{}%", c.render_scale.percent())
    };
    let on_off = |on: bool| if on { "On" } else { "Off" }.to_owned();
    // (section header, title, value, action, is a switch)
    let rows: Vec<(Option<&'static str>, String, String, Option<Action>, bool)> = vec![
        (Some("Graphics"), String::new(), String::new(), None, false),
        (None, "Rendering".into(), c.renderer.label().to_owned(), Some(Action::Renderer), false),
        (None, "Client resolution".into(), scale_text, Some(Action::RenderScale), false),
        (None, "Aspect ratio".into(), c.screen.aspect.label().to_owned(), Some(Action::Aspect), false),
        (None, "Display refresh rate".into(), c.device.display_label().to_owned(), Some(Action::DisplayHz), false),
        (None, "Immersive fullscreen".into(), on_off(c.screen.immersive), Some(Action::Immersive), true),
        (None, "Avoid camera cutout".into(), on_off(c.screen.avoid_cutout), Some(Action::Cutout), true),
        (Some("Performance"), String::new(), String::new(), None, false),
        (None, "Performance hints".into(), if c.device.performance_hints { "On (ADPF)".into() } else { "Off".into() }, Some(Action::PerformanceHints), true),
        (None, "GPU profiling".into(), if c.device.gpu_profile { "On / in report".into() } else { "Off".into() }, Some(Action::GpuProfile), false),
        (None, "Performance recording".into(), if c.record_performance { "On / up to 30 min".into() } else { "Off".into() }, Some(Action::RecordPerformance), false),
        (None, "Diagnostic report".into(), "Export one ZIP to share".into(), Some(Action::ExportDiagnostics), false),
        (None, "Raw performance data".into(), "Advanced / JSONL in Reports".into(), Some(Action::ExportPerformance), false),
        (Some("Accounts & controls"), String::new(), String::new(), None, false),
        (None, "Profile".into(), format!("Profile {}", c.profile), Some(Action::Profile), false),
        (None, "Input".into(), c.input_host.label().to_owned(), Some(Action::InputHost), false),
        (None, "Touch controls".into(), "Layouts: Options > Controls in game".into(), Some(Action::InputHost), false),
        (None, "Orientation".into(), c.orientation.label().to_owned(), Some(Action::Orientation), false),
        (None, "Server".into(), if c.server.is_empty() { "Singleplayer / tap to enter".into() } else { c.server.clone() }, Some(Action::Field("server")), false),
        (None, "Microsoft / Xbox account".into(), "Sign in or edit the offline name".into(), Some(Action::Account), false),
        (None, "Diagnostics".into(), "Last handled errors and reports".into(), Some(Action::Logs), false),
        (Some("About"), String::new(), String::new(), None, false),
        (None, "Version".into(), format!("Horizon Android {}  •  Minecraft Java {}", crate::backend::VERSION, horizon_core::VANILLA_VERSION), None, false),
        (None, "Source & updates".into(), "Open the repository".into(), Some(Action::Repository), false),
    ];
    let (x, w, gap) = (l.content_x, l.content_w, l.gap);
    let top = l.content_top;
    let bottom = l.page_bottom;
    let section_h = design::metric("settings", "section_height");
    let row_h = design::metric("settings", "row_height");
    let total: f32 = rows
        .iter()
        .map(|r| (if r.0.is_some() { section_h } else { row_h }) + gap)
        .sum();
    let maximum = (total - (bottom - top)).max(0.0);
    catalog.scroll = catalog.scroll.clamp(0.0, maximum);
    let view = [top, bottom, maximum];
    let mut offset = 0.0;
    for (section, title, value, action, is_switch) in rows {
        let height = if section.is_some() { section_h } else { row_h };
        let y = top + offset - catalog.scroll;
        if let Some(name) = section {
            if design::visible([x, y, w, section_h], top, bottom) {
                design::section_title(ui, t, [x, y + 4.0, w, 22.0], name);
                ui.rounded(x, y + section_h - 5.0, w, 1.0, 0.0, t.outline());
            }
        } else {
            let r = [x, y, w, row_h - 8.0];
            if let Some(action) = action {
                list_targets.push((r, action));
            }
            if design::visible(r, top, bottom) {
                design::card(ui, t, r, design::radius("card"));
                let title = ui.fit_text(Face::SemiBold, 14.0, &title, w - 40.0);
                ui.text(Face::SemiBold, 14.0, x + 14.0, y + 10.0, t.text(), &title);
                if is_switch {
                    let switch = [x + w - 108.0, y + 16.0, 94.0, 34.0];
                    // `toggle` keeps the original hit area and action.
                    let on = value == "On" || value.starts_with("On");
                    if let Some(action) = action {
                        toggle(ui, targets, switch, on, action);
                    }
                } else {
                    let value = ui.fit_text(Face::Regular, 12.0, &value, w - 56.0);
                    ui.text(Face::Regular, 12.0, x + 14.0, y + 36.0, t.muted(), &value);
                    if let Some(action) = action {
                        design::icon(ui, design::Icon::Chevron, x + w - 32.0, y + 24.0, 20.0, t.dim());
                        targets.push((r, action));
                    }
                }
            }
        }
        offset += height + gap;
    }
    Some(view)
}

impl Launcher {
    /// The same spatial focus algorithm as the game's controller menus.
    pub(super) fn navigate(&mut self, key: &KeyEvent) -> bool {
        self.navigate_key(&key.logical_key)
    }
    pub(super) fn navigate_key(&mut self, key: &winit::keyboard::Key) -> bool {
        use horizon_client::gamepad::{self, Button};
        use winit::keyboard::{Key, NamedKey};
        if self.pending_focus.is_some() {
            return true;
        }
        if self.catalog.editing.is_some() || self.targets.is_empty() {
            return false;
        }
        let button = match key {
            Key::Named(NamedKey::ArrowUp) => Button::DPadUp,
            Key::Named(NamedKey::ArrowDown) => Button::DPadDown,
            Key::Named(NamedKey::ArrowLeft) => Button::DPadLeft,
            Key::Named(NamedKey::ArrowRight) => Button::DPadRight,
            Key::Named(NamedKey::Tab) => {
                self.focus = Some(self.focus.map_or(0, |i| (i + 1) % self.targets.len()));
                return true;
            }
            _ => return false,
        };
        if matches!(button, Button::DPadUp | Button::DPadDown)
            && let Some((_, action)) = self.focus.and_then(|i| self.targets.get(i))
            && let Some(index) = self.list_targets.iter().position(|(_, a)| a == action)
        {
            let rects: Vec<_> = self.list_targets.iter().map(|(r, _)| *r).collect();
            if let Some(next) = gamepad::focus_step_strict(rects[index], button, &rects)
                && let Some(view) = self.list_view
            {
                let (rect, action) = self.list_targets[next];
                let scroll = if let Some(chooser) = &mut self.chooser {
                    &mut chooser.scroll
                } else {
                    &mut self.catalog.scroll
                };
                *scroll = super::gesture::reveal(*scroll, rect, view);
                self.pending_focus = Some(action);
                self.focus = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return true;
            }
        }
        self.focus = Some(
            if let Some(index) = self.focus.filter(|i| *i < self.targets.len()) {
                let rects: Vec<_> = self.targets.iter().map(|(rect, _)| *rect).collect();
                gamepad::focus_step_rect(rects[index], button, &rects).unwrap_or(index)
            } else {
                0
            },
        );
        true
    }

    pub(super) fn draw_pages(&mut self) -> Result<()> {
        let (Some(r), Some(gui)) = (&mut self.renderer, &self.gui) else {
            return Ok(());
        };
        if r.suspended {
            return Ok(());
        }
        self.targets.clear();
        self.list_targets.clear();
        self.list_view = None;
        let hero_pending = !self.hero_ready;
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        // Upload the bundled hero once per launcher atlas. No frame reads a
        // file; the patch is dropped as soon as the surface accepts it.
        if hero_pending && let Some(patch) = &self.hero_patch {
            b.patches.push(patch.clone());
        }
        let mut ui = Ui::mobile(&gui.atlas, &mut b, (0.0, 0.0));
        // OEM navigation bars, display cutouts and the IME viewport are kept
        // out of the layout. Surface and input coordinates stay untouched.
        let mut top_inset = 0.0f32;
        let mut left_inset = 0.0f32;
        let bottom_inset = 0.0f32;
        if let Some(window) = &self.window {
            use winit::platform::android::WindowExtAndroid;
            let rect = window.content_rect();
            let input = winit::platform::android::native_input_scale();
            if rect.bottom > 0 {
                ui.h = ui.h.min(rect.bottom as f32 * input[1] as f32 / ui.density);
            }
            if rect.right > 0 {
                ui.w = ui.w.min(rect.right as f32 * input[0] as f32 / ui.density);
            }
            if rect.top > 0 {
                top_inset = rect.top as f32 * input[1] as f32 / ui.density;
            }
            // Cutouts on the left edge shift the content column instead of
            // centring a narrow desktop container inside the screen.
            if rect.left > 0 {
                left_inset = (rect.left as f32 * input[0] as f32 / ui.density).min(64.0);
            }
        }
        if let Some(bottom) = crate::native_ime::viewport_bottom() {
            ui.h = ui.h.min(bottom as f32 / ui.density);
        }
        let (w, h) = (ui.w, ui.h);
        // ui.w is already clamped to content_rect().right, so there is no
        // remaining right-hand inset to subtract.
        let base = design::Layout::with_insets(w, h, left_inset, top_inset, 0.0, bottom_inset);
        let page = self.catalog.page;
        let theme = design::Theme::new(design::page_alpha(self.page_enter));
        let phase = design::pulse(design::motion("skeleton_ms"));
        // Shift only the page column; header and bottom bar stay fixed.
        let mut layout = base;
        layout.content_x += design::page_shift(self.page_enter);
        let scale = ui.density / ui.batch.scale;
        ui.batch.fill(&gui.atlas, 0.0, 0.0, w * scale, h * scale, theme.bg());
        draw_glows(&mut ui, &theme, w, h);
        draw_header(&mut ui, &theme, &base, &mut self.targets, crate::backend::VERSION);

        let clip_start = ui.vertices();
        if page == Page::Play {
            use horizon_client::official_install::InstallStage;
            let installed = horizon_client::official_install::ready(&self.home);
            let installing = self.worker.is_some();
            let progress = *self.install.lock().unwrap();
            let current = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
            let fraction = if progress.total > 0 {
                progress.done as f32 / progress.total as f32
            } else {
                0.0
            };
            let overall = (current as f32 + fraction.clamp(0.0, 1.0)) / 6.0;
            let mods = self.catalog.local.iter().filter(|e| e.kind == Kind::Mods).count();
            let packs = self.catalog.local.iter().filter(|e| e.kind == Kind::Packs).count();
            let shader_items: Vec<_> = self.catalog.local.iter().filter(|e| e.kind == Kind::Shaders).collect();
            let data = HomeData {
                version: crate::backend::VERSION.to_owned(),
                mc_version: horizon_core::VANILLA_VERSION.to_owned(),
                profile_name: if self.catalog.config.name.is_empty() {
                    "Player".into()
                } else {
                    self.catalog.config.name.clone()
                },
                account_status: self.login.status(&self.catalog.config.name),
                signed_in: self.login.signed_in,
                mods,
                packs,
                shaders_enabled: shader_items.iter().any(|e| e.enabled),
                shader_packs: shader_items.len(),
                installed: self.catalog.local.len(),
                installing,
                install_label: if installing {
                    progress.stage.label().to_owned()
                } else if installed {
                    "Play".into()
                } else if self.error.is_some() {
                    "Retry installation".into()
                } else {
                    "Download Minecraft 26.3".into()
                },
                install_detail: match progress.stage {
                    InstallStage::Download => format!(
                        "{:.1} / {:.1} MiB verified",
                        progress.done as f64 / 1048576.0,
                        progress.total as f64 / 1048576.0
                    ),
                    InstallStage::Controls => format!("{} / {} control files verified", progress.done, progress.total),
                    _ => "Please keep Horizon open".into(),
                },
                install_fraction: overall,
                renderer: self.catalog.config.renderer.label().to_owned(),
                resolution: self.catalog.config.render_scale.percent(),
                memory: design::memory(),
            };
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, layout.home_max_scroll());
            draw_home(
                &mut ui,
                &theme,
                &layout,
                &mut self.targets,
                self.catalog.scroll,
                &data,
                self.hero_patch.is_some(),
            );
            self.list_view = Some([layout.page_top, layout.page_bottom, layout.home_max_scroll()]);
            let _ = data.install_detail;
        } else {
            self.list_view = match page {
                Page::Mods | Page::Packs | Page::Shaders => draw_catalog_page(
                    &mut ui, &theme, &layout, &mut self.targets, &mut self.list_targets,
                    &mut self.catalog, page, phase,
                ),
                Page::Installed => draw_installed_page(
                    &mut ui, &theme, &layout, &mut self.targets, &mut self.list_targets, &mut self.catalog,
                ),
                Page::Settings => draw_settings_page(
                    &mut ui, &theme, &layout, &mut self.targets, &mut self.list_targets, &mut self.catalog,
                ),
                Page::Play => None,
            };
        }
        ui.clip(clip_start, 0.0, layout.page_top, w, layout.page_bottom - layout.page_top);

        // Fixed feedback toast: real status text from the installer, catalog,
        // accounts and diagnostics. Nothing is fabricated here.
        let status = self.error.as_deref().unwrap_or(&self.catalog.status).to_owned();
        if !status.is_empty() {
            let lower = status.to_lowercase();
            let danger = self.error.is_some()
                || lower.contains("fail")
                || lower.contains("could not")
                || lower.starts_with("no ");
            let text = ui.fit_text(Face::Regular, 12.0, &status, base.content_w - 46.0);
            design::toast(
                &mut ui, &theme,
                [base.content_x, base.nav_y - 50.0, base.content_w, 42.0],
                &text, danger,
            );
        }
        // Fixed five-destination bottom bar. Shaders keeps its own page but is
        // reached through the Home card and the content segmented control.
        let active = match page {
            Page::Play => 0,
            Page::Mods => 1,
            Page::Packs | Page::Shaders => 2,
            Page::Installed => 3,
            Page::Settings => 4,
        };
        let labels = design::nav_labels();
        design::navigation(
            &mut ui, &theme, &base, active,
            [design::Icon::Home, design::Icon::Puzzle, design::Icon::Layers, design::Icon::Download, design::Icon::Gear],
            &labels,
        );
        let destinations = [Page::Play, Page::Mods, Page::Packs, Page::Installed, Page::Settings];
        for (i, destination) in destinations.into_iter().enumerate() {
            self.targets.push((base.nav_item(i), Action::Page(destination)));
        }

        if self.login.visible {
            self.list_targets.clear();
            self.list_view = None;
            self.login.draw(
                &mut ui,
                &mut self.targets,
                &self.catalog.config.name,
                self.catalog.editing == Some("name"),
            );
        }
        if let Some(chooser) = &mut self.chooser {
            chooser.draw(&mut ui, &mut self.targets);
            self.list_targets = chooser.navigation(w, h);
            self.list_view = Some(chooser.viewport(w, h));
        }
        if let Some([top, bottom, maximum]) = self.list_view
            && maximum > 0.0
            && bottom > top
        {
            let scroll = self.chooser.as_ref().map_or(self.catalog.scroll, |c| c.scroll);
            let height = bottom - top;
            let thumb = (height * height / (height + maximum)).max(20.0).min(height);
            let y = top + (height - thumb) * (scroll / maximum).clamp(0.0, 1.0);
            ui.rounded(w - 8.0, top, 3.0, height, 1.5, theme.outline());
            ui.rounded(w - 8.0, y, 3.0, thumb, 1.5, design::alpha(theme.primary(), 0.8));
        }
        let caret_visible = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .is_ok_and(|t| t.as_millis() % 1000 < 500);
        if caret_visible && let Some(name) = self.catalog.editing {
            let value = match name {
                "name" => self.catalog.config.name.clone(),
                "server" => self.catalog.config.server.clone(),
                "render_scale" => self.catalog.render_scale_text.clone(),
                _ => self.catalog.query.clone(),
            };
            for ([x, y, width, _], action) in &self.targets {
                if *action == Action::Field(name) || (name == "render_scale" && *action == Action::RenderScale) {
                    let text = ui.fit_text(Face::Medium, 13.0, &value, width - 46.0);
                    let caret_x = x + 42.0 + ui.measure(Face::Medium, 13.0, &text);
                    ui.rounded(caret_x.min(x + width - 10.0), y + 13.0, 1.0, 18.0, 0.0, theme.text());
                }
            }
        }
        if let Some(index) = self.focus
            && let Some(([x, y, tw, th], _)) = self.targets.get(index)
        {
            ui.border(*x - 2.0, *y - 2.0, *tw + 4.0, *th + 4.0, design::radius("chip"), theme.primary());
            self.cursor = (
                f64::from((x + tw / 2.0) * ui.density),
                f64::from((y + th / 2.0) * ui.density),
            );
        }
        // Press and success/failure feedback: a short, low-alpha accent ring.
        let pressed_point = [self.cursor.0 as f32 / ui.density, self.cursor.1 as f32 / ui.density];
        let pressed = self.mouse_down || self.drag.as_ref().is_some_and(|drag| !drag.moved);
        if let Some(([x, y, tw, th], _)) = self.targets.iter().rev().find(|(rect, _)| {
            pressed
                && pressed_point[0] >= rect[0] && pressed_point[0] <= rect[0] + rect[2]
                && pressed_point[1] >= rect[1] && pressed_point[1] <= rect[1] + rect[3]
        }) {
            ui.border(*x - 2.0, *y - 2.0, *tw + 4.0, *th + 4.0, design::radius("chip"), design::alpha(theme.primary(), 0.55));
        }
        if !self.mouse_down
            && self.drag.is_none()
            && let Some((action, time)) = self.feedback
            && time.elapsed() < std::time::Duration::from_millis(design::motion("press_ms") as u64)
            && let Some(([x, y, tw, th], _)) = self.targets.iter().find(|(_, a)| *a == action)
        {
            ui.border(*x - 3.0, *y - 3.0, *tw + 6.0, *th + 6.0, design::radius("chip"), design::alpha(theme.cyan(), 0.7));
        }
        crate::documents::draw_ui(&mut ui);
        ui.finish();

        let rendered = r.render(&game::loading_frame(&b))?;
        if hero_pending && rendered {
            self.hero_ready = true;
        }
        if !rendered && !b.patches.is_empty() {
            // A suspended/lost surface skips the frame, including its glyph
            // and hero uploads. Rebuild them next time.
            horizon_client::ui_theme::reset_atlas();
        }
        Ok(())
    }
    pub(super) fn activate(&mut self, events: &ActiveEventLoop) -> Result<()> {
        if self.session_worker.is_some() {
            return Ok(());
        }
        let Some(r) = &self.renderer else {
            return Ok(());
        };
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let ui = Ui::mobile(&self.gui.as_ref().unwrap().atlas, &mut b, (0.0, 0.0));
        let p = self.focus.and_then(|index| self.targets.get(index)).map_or(
            [
                self.cursor.0 as f32 / ui.density,
                self.cursor.1 as f32 / ui.density,
            ],
            |([x, y, w, h], _)| [x + w / 2.0, y + h / 2.0],
        );
        let action = self
            .targets
            .iter()
            .find(|([x, y, w, h], _)| {
                (*x..*x + *w).contains(&p[0]) && (*y..*y + *h).contains(&p[1])
            })
            .map(|(_, a)| *a);
        if self.catalog.busy() || crate::documents::busy() {
            return Ok(());
        }
        self.feedback = action.map(|action| (action, std::time::Instant::now()));
        self.focus = None;
        if action.is_some() && horizon_client::official_install::ready(&self.home) {
            let audio = self.audio.get_or_insert_with(|| {
                let mut audio = horizon_client::audio::Audio::new(
                    &self.home.join(".local/horizon-client/26.3"),
                );
                let settings = horizon_client::settings::Settings::load(
                    &self
                        .catalog
                        .config
                        .game_home(&self.home)
                        .join("options.txt"),
                );
                audio.master = settings.volumes[0];
                let mut volumes = settings.volumes;
                volumes[0] = 1.0;
                horizon_client::settings::set_source_volumes(volumes);
                audio
            });
            audio.play_button_click();
        }
        match action {
            Some(Action::Immersive) => {
                self.catalog.config.screen.immersive ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Cutout) => {
                self.catalog.config.screen.avoid_cutout ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Aspect) => self.open_chooser(chooser::Kind::Aspect),
            Some(Action::ChooseAspect(aspect)) => {
                self.catalog.config.screen.aspect = aspect;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
                self.chooser = None;
            }
            Some(Action::DisplayHz) => self.open_chooser(chooser::Kind::DisplayHz),
            Some(Action::ChooseDisplayHz(hz)) => {
                self.catalog.config.device.display_hz = hz;
                self.catalog.config.device.sanitize();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
                self.chooser = None;
            }
            Some(Action::PerformanceHints) => {
                self.catalog.config.device.performance_hints ^= true;
                self.catalog.status="ADPF gives Android CPU frame-work hints on supported devices. It does not cap FPS or guarantee a speedup.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }
            Some(Action::GpuProfile) => {
                self.catalog.config.device.gpu_profile ^= true;
                self.catalog.status="Detailed GPU profiling records diagnostics and has overhead. Reports remain on this device.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }

            Some(Action::ModSource(origin)) => {
                self.catalog.config.mod_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::ShaderSource(origin)) => {
                self.catalog.config.shader_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::OpenFolder(kind)) => {
                crate::platform::hide();
                let folder = match kind {
                    horizon_client::content_files::Kind::Mods => "mods",
                    horizon_client::content_files::Kind::Packs => "packs",
                    horizon_client::content_files::Kind::Shaders => "shaders",
                };
                let id = crate::document_store::folder_id(self.catalog.config.profile, folder);
                self.catalog.status = if crate::platform::open_documents(&id) {
                    "Opened in Files. Return here after copying files.".into()
                } else {
                    "Open the system Files app and choose Horizon Android.".into()
                };
            }
            Some(Action::Import(kind)) => {
                crate::platform::hide();
                let home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin(kind, home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ToggleSection(i)) => {
                self.catalog.sections[i] = !self.catalog.sections[i];
                self.catalog.scroll = 0.0;
            }
            Some(Action::ToggleLocal(i)) => self.catalog.toggle_local(&self.home, i),
            Some(Action::DeleteLocal(i)) => {
                self.catalog.pending_delete = self.catalog.local.get(i).cloned();
                if let Some(item) = &self.catalog.pending_delete {
                    self.chooser = Some(super::chooser::Chooser::confirm_delete(item.file.clone()));
                }
            }
            Some(Action::ConfirmDeleteLocal) => {
                self.catalog.remove_local(&self.home);
                self.chooser = None;
            }
            Some(Action::Repository) => {
                self.catalog.status = if crate::platform::open_repository() {
                    "Repository opened in browser."
                } else {
                    "No browser available to open the repository."
                }
                .into();
            }
            Some(Action::Account) => {
                self.login.visible = true;
            }
            Some(Action::CloseLogin) => {
                self.login.hide();
                crate::platform::hide();
                self.catalog.editing = None;
            }
            Some(Action::DeviceSignIn) => self
                .login
                .start_device(&self.catalog.config.microsoft_client_id),
            Some(Action::CopyLoginCode) => self.login.copy_code(),
            Some(Action::CancelLogin) => {
                self.login.close();
                self.login.visible = true;
            }
            Some(Action::ReviewApp) => self.login.review_app(),
            Some(Action::SignIn) => self.login.start(&self.catalog.config.microsoft_client_id),
            Some(Action::SignOut) => self.login.sign_out(),
            Some(Action::LoginLink) => self.login.open_link(),
            Some(Action::Page(page)) => {
                self.catalog.editing = None;
                if crate::platform::active() {
                    crate::platform::hide();
                }
                self.catalog.page = page;
                self.catalog.scroll = 0.0;
                // Short enter animation: presentation state only.
                self.page_enter = Some(std::time::Instant::now());
                self.error = None;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::Play) => self.action(events)?,
            Some(Action::Search) => {
                crate::platform::hide();
                self.catalog.offset = 0;
                self.catalog.search(&self.home);
            }
            Some(Action::FinishNameEdit) => {
                self.catalog.editing = None;
                crate::platform::hide();
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::Field(name)) => {
                if name == "render_scale" {
                    self.catalog.render_scale_text =
                        self.catalog.config.render_scale.percent().to_string();
                    self.chooser = None;
                }
                self.catalog.editing = Some(name);
                let initial = match name {
                    "name" => &self.catalog.config.name,
                    "server" => &self.catalog.config.server,
                    "render_scale" => &self.catalog.render_scale_text,
                    "client_id" => &self.catalog.config.microsoft_client_id,
                    _ => &self.catalog.query,
                };
                crate::platform::show(initial);
            }
            Some(Action::Install(i)) => self.catalog.install(&self.home, i),
            Some(Action::Next) => {
                self.catalog.offset += 12;
                self.catalog.search(&self.home);
            }
            Some(Action::Previous) => {
                self.catalog.offset = self.catalog.offset.saturating_sub(12);
                self.catalog.search(&self.home);
            }
            Some(Action::Profile) => self.open_chooser(chooser::Kind::Profile),
            Some(Action::Renderer) => self.open_chooser(chooser::Kind::Renderer),
            Some(Action::RenderScale) => self.open_chooser(chooser::Kind::RenderScale),
            Some(Action::ChooseRenderScale(scale)) => {
                self.catalog.config.render_scale = scale;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::display::set_scale(scale);
            }
            Some(Action::Orientation) => self.open_chooser(chooser::Kind::Orientation),
            Some(Action::InputHost) => self.open_chooser(chooser::Kind::InputHost),
            Some(Action::ChooseInputHost(mode)) => {
                self.catalog.config.input_host = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::configure_input_mode(mode);
            }
            Some(Action::ChooseOrientation(mode)) => {
                self.catalog.config.orientation = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::platform::set_orientation(mode);
            }
            Some(Action::ChooseRenderer(mode)) => {
                self.catalog.config.renderer = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
            }
            Some(Action::ChooseProfile(profile)) => {
                std::fs::create_dir_all(if profile == 0 {
                    self.home.clone()
                } else {
                    self.home
                        .join("profiles")
                        .join(format!("profile-{profile}"))
                })?;
                self.catalog.config.profile = profile;
                self.catalog.config.save(&self.home)?;
                self.catalog.refresh(&self.home);
                self.chooser = None;
            }
            Some(Action::Dismiss) => self.chooser = None,
            Some(Action::Logs) => {
                self.catalog.status = crate::diagnostics::summary(&self.home);
                if let Ok(error) = std::fs::read_to_string(self.home.join("android-error.txt")) {
                    self.catalog.status = format!(
                        "Last handled error: {}. {}",
                        crate::diagnostics::redact(&error),
                        self.catalog.status
                    );
                }
            }
            Some(Action::ExportDiagnostics) if self.diagnostic_export.is_none() => {
                let home = self.home.clone();
                let (sender, receiver) = mpsc::channel();
                self.diagnostic_export = Some(receiver);
                self.catalog.status =
                    "Preparing local diagnostic ZIP. No upload; review before sharing.".into();
                std::thread::spawn(move || {
                    let result = crate::diagnostics::export(&home).map(|p| {
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned()
                    });
                    let _ = sender.send(result);
                });
            }
            Some(Action::ExportDiagnostics) => {}
            Some(Action::RecordPerformance) => {
                self.catalog.config.record_performance = !self.catalog.config.record_performance;
                if !self.catalog.config.record_performance {
                    crate::performance::stop();
                }
                self.catalog.config.save(&self.home)?;
                self.catalog.status = if self.catalog.config.record_performance {
                    "Recording starts on Play. Detailed timings add overhead. Return here to save the report."
                } else { "Performance recording off." }.into();
            }
            Some(Action::ExportPerformance) => {
                self.catalog.status = match crate::performance::export(&self.home) {
                    Ok(()) => "Reports opened in Files. Copy or share a report from there.".into(),
                    Err(e) => e,
                };
            }
            None => {}
        }
        Ok(())
    }
    pub(super) fn ui_density(&self) -> f32 {
        let (Some(r), Some(gui)) = (&self.renderer, &self.gui) else {
            return 1.0;
        };
        let mut batch = HudBatch::new([r.config.width, r.config.height]);
        Ui::mobile(&gui.atlas, &mut batch, (0.0, 0.0)).density
    }

    pub(super) fn touch_event(&mut self, events: &ActiveEventLoop, t: Touch) -> Result<()> {
        self.focus = None;
        self.pending_focus = None;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        self.cursor = (t.location.x, t.location.y);
        match t.phase {
            TouchPhase::Started => {
                self.drag = Some(super::gesture::Drag::new(t.location.x, t.location.y));
            }
            TouchPhase::Moved => {
                let density = self.ui_density() as f64;
                if let Some(drag) = &mut self.drag {
                    let delta = drag.update(t.location.x, t.location.y, density);
                    if drag.moved {
                        if let Some(chooser) = &mut self.chooser {
                            chooser.scroll = (chooser.scroll + delta).max(0.0);
                        } else if self
                            .list_view
                            .is_some_and(|view| drag.origin_y() / density >= view[0] as f64)
                        {
                            self.catalog.scroll = (self.catalog.scroll + delta).max(0.0);
                        }
                    }
                }
            }
            TouchPhase::Ended => {
                if self.drag.take().is_some_and(|drag| !drag.moved) {
                    self.activate(events)?;
                }
            }
            TouchPhase::Cancelled => {
                self.drag = None;
            }
        }
        Ok(())
    }
    pub(super) fn type_key(&mut self, key: KeyEvent) {
        if key.physical_key == PhysicalKey::Code(KeyCode::Escape)
            || key.logical_key
                == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack)
        {
            if self.login.visible {
                self.login.hide();
                crate::platform::hide();
                self.catalog.editing = None;
                return;
            }
            if self.chooser.take().is_some() {
                return;
            }
            crate::platform::hide();
            self.catalog.editing = None;
            let _ = self.catalog.config.save(&self.home);
            return;
        }
        if self.catalog.editing.is_none() {
            return;
        }
        let out = match key.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => Some(Output::Backspace),
            PhysicalKey::Code(KeyCode::Enter) => {
                crate::platform::hide();
                Some(Output::Enter)
            }
            _ => key.text.map(|s| Output::Text(s.into())),
        };
        if let Some(out) = out {
            self.catalog.edit(out, &self.home);
        }
    }
}
