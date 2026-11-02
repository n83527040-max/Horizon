//! The Horizon Android launcher's mobile-first visual language.
//!
//! This module is presentation only. It knows nothing about Minecraft, the
//! Rust engine, accounts, installs or files: `launcher_pages.rs` keeps every
//! one of those connections. The tokens below come from `ui_spec.json`, which
//! the Python layout verifier and the QA preview read too, so a change to a
//! radius or a palette entry is checked in one place.
use horizon_client::ui_theme::{Face, Ui, color};
use serde_json::Value;
use std::{
    sync::{Mutex, OnceLock},
    time::Instant,
};

const SPEC_JSON: &str = include_str!("ui_spec.json");

fn spec() -> &'static Value {
    static SPEC: OnceLock<Value> = OnceLock::new();
    SPEC.get_or_init(|| serde_json::from_str(SPEC_JSON).expect("ui_spec.json is valid JSON"))
}
fn number(section: &str, key: &str) -> f32 {
    spec()[section][key].as_f64().unwrap_or(0.0) as f32
}
fn hex(name: &str) -> u32 {
    let text = spec()["palette"][name].as_str().unwrap_or("#FFFFFF");
    let rgb = u32::from_str_radix(text.trim_start_matches('#'), 16).unwrap_or(0xFFFFFF);
    color::hex(rgb, 255)
}
/// Replace the alpha byte of a HUD colour (ABGR encoded).
pub fn alpha(c: u32, a: f32) -> u32 {
    let base = (c >> 24) as f32;
    (c & 0x00ff_ffff) | ((base * a.clamp(0.0, 1.0)).round() as u32) << 24
}

/// One frame's palette. `alpha` drives the short page-enter fade; pressing
/// feedback and skeleton shimmer use their own multipliers.
#[derive(Clone, Copy)]
pub struct Theme {
    pub alpha: f32,
}
impl Theme {
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
        }
    }
    pub fn tint(&self, name: &str, strength: f32) -> u32 {
        alpha(hex(name), self.alpha * strength)
    }
    pub fn bg(&self) -> u32 {
        self.tint("bg", 1.0)
    }
    pub fn card(&self) -> u32 {
        self.tint("card", 1.0)
    }
    pub fn card_alt(&self) -> u32 {
        self.tint("card_alt", 1.0)
    }
    pub fn glass(&self) -> u32 {
        self.tint("glass", 0.94)
    }
    pub fn outline(&self) -> u32 {
        self.tint("outline", 0.9)
    }
    pub fn text(&self) -> u32 {
        self.tint("text", 1.0)
    }
    pub fn muted(&self) -> u32 {
        self.tint("muted", 1.0)
    }
    pub fn dim(&self) -> u32 {
        self.tint("muted", 0.58)
    }
    pub fn primary(&self) -> u32 {
        self.tint("primary", 1.0)
    }
    pub fn secondary(&self) -> u32 {
        self.tint("secondary", 1.0)
    }
    pub fn cyan(&self) -> u32 {
        self.tint("cyan", 1.0)
    }
    pub fn success(&self) -> u32 {
        self.tint("success", 1.0)
    }
    pub fn danger(&self) -> u32 {
        self.tint("danger", 1.0)
    }
}
pub fn radius(name: &str) -> f32 {
    number("radius", name)
}
pub fn space(name: &str) -> f32 {
    number("spacing", name)
}
pub fn metric(section: &str, key: &str) -> f32 {
    number(section, key)
}
pub fn motion(name: &str) -> f32 {
    number("motion", name)
}
pub fn nav_labels() -> Vec<String> {
    let Some(items) = spec()["nav_items"].as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

// ---------------------------------------------------------------------------
// Responsive layout. All values are dp, the same unit `Ui` draws in.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub w: f32,
    pub h: f32,
    pub pad: f32,
    pub gap: f32,
    pub content_x: f32,
    pub content_w: f32,
    pub safe_top: f32,
    pub safe_bottom: f32,
    pub header_y: f32,
    pub header_h: f32,
    pub subheader_h: f32,
    pub page_top: f32,
    pub content_top: f32,
    pub page_bottom: f32,
    pub nav_y: f32,
    pub nav_h: f32,
    pub hero_h: f32,
}
impl Layout {
    /// `top_inset`/`bottom_inset` come from Android's content rect and the IME
    /// viewport, converted to dp by the caller. Nothing is allowed under the
    /// fixed bottom navigation or above the status-bar inset.
    pub fn new(w: f32, h: f32, top_inset: f32, bottom_inset: f32) -> Self {
        let pad = space("pad").max(12.0);
        let gap = space("gap").max(8.0);
        let max_w = spec()["max_content_width"].as_f64().unwrap_or(560.0) as f32;
        let content_w = (w - 2.0 * pad).min(max_w).max(240.0);
        let content_x = (w - content_w) * 0.5;
        let safe_top = top_inset.max(0.0) + space("safe_top");
        let safe_bottom = bottom_inset.max(0.0) + space("safe_bottom");
        let header_h = metric("header", "height");
        let header_y = safe_top;
        let subheader_h = metric("subheader", "height");
        let nav_h = metric("nav", "height");
        let nav_y = (h - nav_h - safe_bottom).max(header_y + header_h + 80.0);
        let page_top = header_y + header_h + 6.0;
        let content_top = page_top + subheader_h;
        let page_bottom = nav_y - 6.0;
        let hero_h = metric("hero", "height")
            .clamp(metric("hero", "min"), metric("hero", "max"))
            .min((h * 0.26).max(metric("hero", "min")));
        Self {
            w,
            h,
            pad,
            gap,
            content_x,
            content_w,
            safe_top,
            safe_bottom,
            header_y,
            header_h,
            subheader_h,
            page_top,
            content_top,
            page_bottom,
            nav_y,
            nav_h,
            hero_h,
        }
    }
    /// Total height of the scrollable Home column before any scroll offset.
    pub fn home_height(&self) -> f32 {
        let gap = self.gap;
        let pair = metric("version", "height");
        let features = metric("feature", "height");
        self.hero_h
            + gap
            + metric("play", "height")
            + gap
            + pair
            + space("section")
            + 22.0
            + gap
            + (features * 2.0 + gap)
            + gap
            + metric("performance", "height")
            + gap
            + 44.0
            + 16.0
    }
    pub fn home_max_scroll(&self) -> f32 {
        (self.home_height() - (self.page_bottom - self.page_top)).max(0.0)
    }
    pub fn home(&self, scroll: f32) -> HomeRects {
        let x = self.content_x;
        let w = self.content_w;
        let gap = self.gap;
        let mut y = self.page_top - scroll;
        let hero = [x, y, w, self.hero_h];
        y += self.hero_h + gap;
        let play = [x, y, w, metric("play", "height")];
        y += metric("play", "height") + gap;
        let pair_w = (w - gap) * 0.5;
        let version = [x, y, pair_w, metric("version", "height")];
        let profile = [x + pair_w + gap, y, pair_w, metric("version", "height")];
        y += metric("version", "height") + space("section");
        let manage_label = [x, y, w, 22.0];
        y += 22.0 + gap;
        let feature_h = metric("feature", "height");
        let card_w = (w - gap) * 0.5;
        let features = [
            [x, y, card_w, feature_h],
            [x + card_w + gap, y, card_w, feature_h],
            [x, y + feature_h + gap, card_w, feature_h],
            [x + card_w + gap, y + feature_h + gap, card_w, feature_h],
        ];
        y += feature_h * 2.0 + gap + gap;
        let performance = [x, y, w, metric("performance", "height")];
        y += metric("performance", "height") + gap;
        let toast = [x, y, w, 44.0];
        HomeRects {
            hero,
            play,
            version,
            profile,
            manage_label,
            features,
            performance,
            toast,
        }
    }
    pub fn nav_item(&self, index: usize) -> [f32; 4] {
        let width = self.w / 5.0;
        [index as f32 * width, self.nav_y, width, self.nav_h - self.safe_bottom]
    }
    /// A full-width row inside the content column.
    pub fn row(&self, y: f32, h: f32) -> [f32; 4] {
        [self.content_x, y, self.content_w, h]
    }
    pub fn half(&self, y: f32, h: f32, right: bool) -> [f32; 4] {
        let w = (self.content_w - self.gap) * 0.5;
        [
            if right {
                self.content_x + w + self.gap
            } else {
                self.content_x
            },
            y,
            w,
            h,
        ]
    }
    pub fn third(&self, y: f32, h: f32, index: usize) -> [f32; 4] {
        let w = (self.content_w - 2.0 * self.gap) / 3.0;
        [
            self.content_x + index as f32 * (w + self.gap),
            y,
            w,
            h,
        ]
    }
}
#[derive(Clone, Copy, Debug)]
pub struct HomeRects {
    pub hero: [f32; 4],
    pub play: [f32; 4],
    pub version: [f32; 4],
    pub profile: [f32; 4],
    pub manage_label: [f32; 4],
    pub features: [[f32; 4]; 4],
    pub performance: [f32; 4],
    pub toast: [f32; 4],
}
pub fn visible(r: [f32; 4], top: f32, bottom: f32) -> bool {
    r[1] + r[3] > top && r[1] < bottom
}

// ---------------------------------------------------------------------------
// Motion. All effects are cheap: a few alpha multiplies per frame, no timers
// are spawned and nothing runs while the game owns the renderer.
// ---------------------------------------------------------------------------

pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}
pub fn page_alpha(since: Option<Instant>) -> f32 {
    let Some(start) = since else { return 1.0 };
    let duration = motion("page_ms").max(1.0) / 1000.0;
    let t = start.elapsed().as_secs_f32() / duration;
    0.72 + 0.28 * ease_out(t)
}
pub fn page_shift(since: Option<Instant>) -> f32 {
    let Some(start) = since else { return 0.0 };
    let duration = motion("page_ms").max(1.0) / 1000.0;
    let t = start.elapsed().as_secs_f32() / duration;
    10.0 * (1.0 - ease_out(t))
}
/// 0..1 shimmer used by skeletons and the soft glow, from a monotonic clock.
pub fn pulse(period_ms: f32) -> f32 {
    let ms = Instant::now().elapsed().as_millis() as f32;
    // `elapsed` on a fresh Instant is ~0; use the wall clock instead so the
    // phase is shared by every widget drawn in one frame.
    let wall = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f32)
        .unwrap_or(ms);
    let p = (wall % period_ms.max(1.0)) / period_ms.max(1.0);
    (p * std::f32::consts::TAU).sin() * 0.5 + 0.5
}

// ---------------------------------------------------------------------------
// Real device memory. Android is Linux: /proc is the engine's own source for
// diagnostics. Cached for two seconds so drawing a card never reads a file
// every frame. Nothing is invented when the kernel does not expose a value.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
pub struct Memory {
    pub rss_mb: Option<u32>,
    pub peak_mb: Option<u32>,
    pub device_mb: Option<u32>,
}
fn kib(text: &str, key: &str) -> Option<u32> {
    let line = text.lines().find(|l| l.starts_with(key))?;
    let value: u32 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(value / 1024)
}
fn read_memory() -> Memory {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    Memory {
        rss_mb: kib(&status, "VmRSS:"),
        peak_mb: kib(&status, "VmHWM:"),
        device_mb: kib(&meminfo, "MemTotal:"),
    }
}
pub fn memory() -> Memory {
    static CACHE: OnceLock<Mutex<(Instant, Memory)>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new((Instant::now() - std::time::Duration::from_secs(9), Memory::default())));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if guard.0.elapsed().as_secs() >= 2 {
        guard.0 = Instant::now();
        guard.1 = read_memory();
    }
    guard.1
}
pub fn gib(mb: u32) -> String {
    if mb >= 1024 {
        format!("{:.1} GB", mb as f32 / 1024.0)
    } else {
        format!("{mb} MB")
    }
}

// ---------------------------------------------------------------------------
// Icons. One unified 24dp stroke language, drawn from rounded rects so they
// need no extra atlas slots, no downloads and no per-frame geometry uploads.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum Icon {
    Home,
    Puzzle,
    Layers,
    Cube,
    Download,
    Gauge,
    Gear,
    Play,
    Chevron,
    Search,
    Folder,
    User,
    Cpu,
    Shield,
    Check,
    Trash,
    Sparkle,
    Info,
    Refresh,
}
fn bar(ui: &mut Ui, x: f32, y: f32, w: f32, h: f32, c: u32) {
    ui.rounded(x, y, w, h, h * 0.5, c);
}
fn disc(ui: &mut Ui, x: f32, y: f32, d: f32, c: u32) {
    ui.rounded(x, y, d, d, d * 0.5, c);
}
fn ring(ui: &mut Ui, x: f32, y: f32, d: f32, t: f32, c: u32) {
    ui.border(x, y, d, d, d * 0.5, c);
    let _ = t;
}
/// `size` is the square box; every icon is centred inside it.
pub fn icon(ui: &mut Ui, which: Icon, x: f32, y: f32, size: f32, c: u32) {
    let s = size;
    let u = s / 24.0;
    let px = |v: f32| x + v * u;
    let py = |v: f32| y + v * u;
    let w = |v: f32| v * u;
    let t = (1.9 * u).max(1.4);
    match which {
        Icon::Home => {
            bar(ui, px(5.0), py(11.0), w(14.0), t, c);
            bar(ui, px(7.0), py(11.0), w(10.0), w(9.0), c);
            ui.rounded(px(10.0), py(15.0), w(4.0), w(5.0), w(0.6), c);
            for i in 0..3 {
                bar(ui, px(3.0 + i as f32 * 3.0), py(7.0 + i as f32 * 1.7), w(18.0 - i as f32 * 6.0), t, c);
            }
        }
        Icon::Puzzle => {
            ui.rounded(px(4.0), py(7.0), w(10.0), w(10.0), w(2.0), c);
            ui.rounded(px(12.0), py(7.0), w(8.0), w(10.0), w(2.0), c);
            disc(ui, px(7.0), py(3.5), w(4.5), c);
            disc(ui, px(14.0), py(15.5), w(4.5), c);
            ui.rounded(px(9.0), py(9.0), w(6.0), w(6.0), w(1.0), color::hex(0x111A35, 255));
        }
        Icon::Layers => {
            for i in 0..3 {
                let yy = py(5.0 + i as f32 * 5.0);
                let inset = i as f32 * 1.5;
                bar(ui, px(3.0 + inset), yy, w(18.0 - inset * 2.0), t, c);
            }
            bar(ui, px(5.0), py(6.0), t, w(13.0), c);
            bar(ui, px(17.0), py(6.0), t, w(13.0), c);
        }
        Icon::Cube => {
            ui.border(px(4.0), py(7.0), w(11.0), w(11.0), w(1.5), c);
            ui.border(px(9.0), py(4.0), w(11.0), w(11.0), w(1.5), c);
            bar(ui, px(4.0), py(7.0), w(5.0), t, c);
            bar(ui, px(15.0), py(15.0), w(5.0), t, c);
        }
        Icon::Download => {
            bar(ui, px(4.0), py(17.0), w(16.0), t, c);
            bar(ui, px(6.0), py(17.0), t, w(3.0), c);
            bar(ui, px(16.0), py(17.0), t, w(3.0), c);
            bar(ui, px(11.0), py(4.0), t, w(10.0), c);
            bar(ui, px(7.5), py(10.0), w(5.0), t, c);
            bar(ui, px(12.0), py(10.0), w(5.0), t, c);
        }
        Icon::Gauge => {
            for i in 0..7 {
                let a = std::f32::consts::PI * (0.15 + i as f32 * 0.115);
                disc(ui, px(11.5 + a.cos() * 7.5), py(13.0 - a.sin() * 7.5), w(1.8), c);
            }
            bar(ui, px(11.0), py(12.0), w(8.0), t, c);
            disc(ui, px(10.0), py(11.0), w(3.5), c);
        }
        Icon::Gear => {
            ring(ui, px(5.0), py(5.0), w(14.0), t, c);
            disc(ui, px(9.5), py(9.5), w(5.0), c);
            for (dx, dy) in [(11.0, 2.0), (2.0, 11.0), (20.0, 11.0), (11.0, 20.0)] {
                disc(ui, px(dx), py(dy), w(2.6), c);
            }
            disc(ui, px(10.5), py(10.5), w(3.0), color::hex(0x111A35, 255));
        }
        Icon::Play => {
            for i in 0..5 {
                let width = w(12.0 - i as f32 * 2.6);
                bar(ui, px((24.0 - (12.0 - i as f32 * 2.6)) * 0.5), py(5.0 + i as f32 * 2.8), width, t, c);
            }
        }
        Icon::Chevron => {
            for i in 0..4 {
                bar(ui, px(9.0 + i as f32 * 1.4), py(8.0 + i as f32 * 2.0), w(3.0), t, c);
            }
            for i in 0..4 {
                bar(ui, px(12.0 - i as f32 * 1.4), py(14.0 + i as f32 * 2.0), w(3.0), t, c);
            }
        }
        Icon::Search => {
            ring(ui, px(4.0), py(4.0), w(11.0), t, c);
            bar(ui, px(14.0), py(14.0), w(6.0), t, c);
        }
        Icon::Folder => {
            ui.rounded(px(3.0), py(7.0), w(18.0), w(12.0), w(2.0), c);
            bar(ui, px(3.0), py(5.0), w(7.0), t, c);
            ui.rounded(px(6.0), py(10.0), w(12.0), w(6.0), w(1.0), color::hex(0x111A35, 255));
        }
        Icon::User => {
            disc(ui, px(8.0), py(4.0), w(8.0), c);
            ui.rounded(px(5.0), py(13.0), w(14.0), w(8.0), w(5.0), c);
        }
        Icon::Cpu => {
            ui.border(px(5.0), py(5.0), w(14.0), w(14.0), w(2.0), c);
            ui.rounded(px(9.0), py(9.0), w(6.0), w(6.0), w(1.0), c);
            for i in 0..3 {
                bar(ui, px(2.0), py(8.0 + i as f32 * 4.0), w(3.0), t, c);
                bar(ui, px(19.0), py(8.0 + i as f32 * 4.0), w(3.0), t, c);
                bar(ui, px(8.0 + i as f32 * 4.0), py(2.0), t, w(3.0), c);
                bar(ui, px(8.0 + i as f32 * 4.0), py(19.0), t, w(3.0), c);
            }
        }
        Icon::Shield => {
            bar(ui, px(4.0), py(5.0), w(16.0), t, c);
            ui.rounded(px(4.0), py(5.0), w(16.0), w(10.0), w(2.0), c);
            for i in 0..3 {
                bar(ui, px(6.0 + i as f32 * 2.0), py(15.0 + i as f32 * 2.5), w(12.0 - i as f32 * 4.0), t, c);
            }
        }
        Icon::Check => {
            bar(ui, px(4.0), py(12.0), w(5.0), t, c);
            bar(ui, px(8.0), py(15.0), w(5.0), t, c);
            bar(ui, px(12.0), py(10.0), w(8.0), t, c);
        }
        Icon::Trash => {
            bar(ui, px(4.0), py(5.0), w(16.0), t, c);
            bar(ui, px(10.0), py(3.0), w(4.0), t, c);
            ui.rounded(px(6.0), py(7.0), w(12.0), w(13.0), w(1.5), c);
            bar(ui, px(10.0), py(10.0), t, w(7.0), c);
            bar(ui, px(13.0), py(10.0), t, w(7.0), c);
        }
        Icon::Sparkle => {
            disc(ui, px(10.0), py(3.0), w(3.0), c);
            disc(ui, px(10.0), py(18.0), w(3.0), c);
            disc(ui, px(3.0), py(10.5), w(3.0), c);
            disc(ui, px(18.0), py(10.5), w(3.0), c);
            disc(ui, px(10.0), py(10.5), w(4.0), c);
        }
        Icon::Info => {
            ring(ui, px(4.0), py(4.0), w(16.0), t, c);
            disc(ui, px(11.0), py(7.0), w(2.4), c);
            bar(ui, px(11.0), py(11.0), t, w(7.0), c);
        }
        Icon::Refresh => {
            ring(ui, px(4.0), py(4.0), w(16.0), t, c);
            bar(ui, px(16.0), py(4.0), w(6.0), t, c);
            bar(ui, px(18.0), py(4.0), t, w(5.0), c);
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

pub fn card(ui: &mut Ui, t: &Theme, r: [f32; 4], radius: f32) {
    ui.rounded(r[0], r[1], r[2], r[3], radius, t.card());
    ui.border(r[0], r[1], r[2], r[3], radius, t.outline());
}
pub fn card_alt(ui: &mut Ui, t: &Theme, r: [f32; 4], radius: f32) {
    ui.rounded(r[0], r[1], r[2], r[3], radius, t.card_alt());
    ui.border(r[0], r[1], r[2], r[3], radius, t.outline());
}
/// A card with a very light top-to-bottom glassmorphism sheen.
pub fn glass_card(ui: &mut Ui, t: &Theme, r: [f32; 4], radius: f32) {
    card(ui, t, r, radius);
    let inset = radius.min(10.0);
    ui.gradient(
        r[0] + inset,
        r[1] + 1.0,
        r[2] - inset * 2.0,
        22.0,
        alpha(hex("primary"), t.alpha * 0.09),
        alpha(hex("card"), 0.0),
    );
}
pub fn gradient_button(ui: &mut Ui, t: &Theme, r: [f32; 4], glow: f32) {
    let rad = radius("button").min(r[3] * 0.5);
    ui.rounded(r[0], r[1], r[2], r[3], rad, t.secondary());
    // The gradient fills the straight middle; the rounded base shows at the
    // corners, which is enough for a soft diagonal-looking blend.
    ui.gradient(
        r[0] + rad,
        r[1],
        r[2] - rad * 2.0,
        r[3],
        alpha(hex("primary"), t.alpha),
        alpha(hex("secondary"), t.alpha),
    );
    ui.border(r[0], r[1], r[2], r[3], rad, alpha(hex("primary"), t.alpha * 0.55));
    if glow > 0.0 {
        ui.border(
            r[0] - 3.0,
            r[1] - 3.0,
            r[2] + 6.0,
            r[3] + 6.0,
            rad + 3.0,
            alpha(hex("primary"), t.alpha * 0.28 * glow),
        );
    }
}
pub fn chip(ui: &mut Ui, t: &Theme, r: [f32; 4], active: bool, accent: &str) {
    let rad = radius("chip");
    ui.rounded(r[0], r[1], r[2], r[3], rad, if active { alpha(hex(accent), t.alpha * 0.32) } else { t.card_alt() });
    ui.border(r[0], r[1], r[2], r[3], rad, if active { alpha(hex(accent), t.alpha) } else { t.outline() });
}
pub fn badge(ui: &mut Ui, t: &Theme, r: [f32; 4], text: &str) {
    let rad = r[3] * 0.5;
    ui.rounded(r[0], r[1], r[2], r[3], rad, alpha(hex("primary"), t.alpha * 0.24));
    ui.border(r[0], r[1], r[2], r[3], rad, alpha(hex("primary"), t.alpha * 0.8));
    let label = ui.fit_text(Face::Medium, 11.0, text, r[2] - 8.0);
    let width = ui.measure(Face::Medium, 11.0, &label);
    ui.text(Face::Medium, 11.0, r[0] + (r[2] - width) * 0.5, r[1] + 7.0, t.text(), &label);
}
pub fn section_title(ui: &mut Ui, t: &Theme, r: [f32; 4], text: &str) {
    let label = ui.fit_text(Face::SemiBold, 15.0, text, r[2]);
    ui.text(Face::SemiBold, 15.0, r[0], r[1], t.text(), &label);
}
pub fn progress(ui: &mut Ui, t: &Theme, r: [f32; 4], fraction: f32, color_name: &str) {
    let rad = r[3] * 0.5;
    ui.rounded(r[0], r[1], r[2], r[3], rad, t.card_alt());
    let fill = (r[2] * fraction.clamp(0.0, 1.0)).max(0.0);
    if fill > 0.0 {
        ui.rounded(r[0], r[1], fill, r[3], rad, alpha(hex(color_name), t.alpha * 0.95));
    }
}
pub fn skeleton(ui: &mut Ui, t: &Theme, r: [f32; 4], phase: f32) {
    card(ui, t, r, radius("card"));
    let shimmer = 0.18 + 0.18 * phase;
    let x = r[0] + 14.0;
    let w = r[2] - 28.0;
    ui.rounded(x, r[1] + 18.0, w * 0.55, 12.0, 6.0, alpha(hex("muted"), t.alpha * shimmer));
    ui.rounded(x, r[1] + 40.0, w * 0.82, 9.0, 5.0, alpha(hex("muted"), t.alpha * shimmer * 0.75));
    ui.rounded(x, r[1] + 58.0, w * 0.35, 9.0, 5.0, alpha(hex("muted"), t.alpha * shimmer * 0.6));
}
pub fn toast(ui: &mut Ui, t: &Theme, r: [f32; 4], text: &str, danger: bool) {
    let accent = if danger { "danger" } else { "primary" };
    let rad = radius("chip");
    ui.rounded(r[0], r[1], r[2], r[3], rad, alpha(hex(accent), t.alpha * 0.16));
    ui.border(r[0], r[1], r[2], r[3], rad, alpha(hex(accent), t.alpha * 0.7));
    let label = ui.fit_text(Face::Regular, 12.0, text, r[2] - 46.0);
    icon(ui, if danger { Icon::Info } else { Icon::Check }, r[0] + 12.0, r[1] + 11.0, 20.0, alpha(hex(accent), t.alpha));
    ui.text(Face::Regular, 12.0, r[0] + 40.0, r[1] + 12.0, t.text(), &label);
}
/// Fixed dark-glass bottom navigation with a short active pill.
pub fn navigation(
    ui: &mut Ui,
    t: &Theme,
    layout: &Layout,
    active: usize,
    icons: [Icon; 5],
    labels: &[String],
) {
    let r = [0.0, layout.nav_y, layout.w, layout.nav_h];
    ui.rounded(r[0], r[1], r[2], r[3], radius("nav"), t.glass());
    ui.border(r[0], r[1], r[2], r[3], radius("nav"), t.outline());
    ui.gradient(r[0] + radius("nav"), r[1], r[2] - radius("nav") * 2.0, 18.0, alpha(hex("primary"), t.alpha * 0.12), alpha(hex("glass"), 0.0));
    let icon_size = metric("nav", "icon");
    let label_size = metric("nav", "label");
    for i in 0..5 {
        let item = layout.nav_item(i);
        let c = if i == active { t.primary() } else { t.dim() };
        let ix = item[0] + (item[2] - icon_size) * 0.5;
        icon(ui, icons[i], ix, item[1] + 12.0, icon_size, c);
        let label = labels.get(i).map(String::as_str).unwrap_or("");
        let fitted = ui.fit_text(Face::Medium, label_size, label, item[2] - 6.0);
        let width = ui.measure(Face::Medium, label_size, &fitted);
        ui.text(Face::Medium, label_size, item[0] + (item[2] - width) * 0.5, item[1] + 39.0, c, &fitted);
        if i == active {
            ui.rounded(item[0] + item[2] * 0.32, item[1] + 3.0, item[2] * 0.36, 3.0, 2.0, t.primary());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_test_screen_keeps_cards_inside_the_viewport_and_nav_visible() {
        for (w, h) in [(360.0, 800.0), (393.0, 873.0), (432.0, 960.0), (480.0, 768.0), (960.0, 540.0)] {
            let l = Layout::new(w, h, 0.0, 0.0);
            let home = l.home(0.0);
            let rects = [
                home.hero, home.play, home.version, home.profile, home.performance, home.toast,
                home.features[0], home.features[1], home.features[2], home.features[3],
            ];
            for r in rects {
                assert!(r[0] >= 0.0 && r[0] + r[2] <= w + 0.01, "horizontal overflow at {w}x{h}");
                assert!(r[2] > 0.0 && r[3] > 0.0);
            }
            assert!(l.nav_y + l.nav_h <= h + 0.01);
            assert!(l.page_bottom <= l.nav_y);
            assert!(l.home_max_scroll() >= 0.0);
            for i in 0..5 {
                let item = l.nav_item(i);
                assert!(item[0] >= 0.0 && item[0] + item[2] <= w + 0.01);
                assert!(item[1] + item[3] <= h + 0.01);
            }
        }
    }
    #[test]
    fn feature_cards_never_overlap() {
        let l = Layout::new(360.0, 800.0, 0.0, 0.0);
        let home = l.home(0.0);
        for a in 0..4 {
            for b in (a + 1)..4 {
                let (p, q) = (home.features[a], home.features[b]);
                let separate = p[0] + p[2] <= q[0] || q[0] + q[2] <= p[0]
                    || p[1] + p[3] <= q[1] || q[1] + q[3] <= p[1];
                assert!(separate);
            }
        }
    }
    #[test]
    fn memory_parser_uses_real_proc_values() {
        let status = "Name:\thori\nVmRSS:\t 4194304 kB\nVmHWM:\t 6291456 kB\n";
        let meminfo = "MemTotal:       8388608 kB\nMemFree: 1 kB\n";
        assert_eq!(kib(status, "VmRSS:"), Some(4096));
        assert_eq!(kib(status, "VmHWM:"), Some(6144));
        assert_eq!(kib(meminfo, "MemTotal:"), Some(8192));
        assert_eq!(kib(status, "VmSize:"), None);
    }
}
