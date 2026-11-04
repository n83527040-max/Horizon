//! Multitouch input using the engine's existing controller actions.
use crate::gamepad::{Button, PadState};
use crate::hud::{HudAtlas, HudBatch, rgba};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
const HOLD_DELAY: Duration = Duration::from_millis(350);
mod layout;
mod ownership;
pub use layout::{
    CustomButton, CustomIcon, CustomVisibility, Control, Mode, Placement, Settings,
};
pub use ownership::InputOwners;
/// Entity-placement items retain short-tap placement, like building blocks.
pub fn item_places_entity_on_tap(item: &str) -> bool {
    let Some(id) = item.strip_prefix("minecraft:") else {
        return false;
    };
    id.ends_with("_boat")
        || id.ends_with("_raft")
        || id.ends_with("_spawn_egg")
        || id == "minecart"
        || id.ends_with("_minecart")
        || matches!(
            id,
            "armor_stand" | "item_frame" | "glow_item_frame" | "painting" | "end_crystal"
        )
}

/// Surface-targeted actions should not wait for a long press.
pub fn item_uses_on_tap(item: &str) -> bool {
    item_places_entity_on_tap(item)
        || item
            .strip_prefix("minecraft:")
            .is_some_and(|id| id.ends_with("_bucket") && id != "milk_bucket")
        || matches!(
            item,
            "minecraft:bone_meal"
                | "minecraft:flint_and_steel"
                | "minecraft:fire_charge"
                | "minecraft:honeycomb"
                | "minecraft:glass_bottle"
                | "minecraft:bucket"
                | "minecraft:water_bucket"
                | "minecraft:lava_bucket"
                | "minecraft:powder_snow_bucket"
        )
}
pub fn projectile_crosshair(item: &str) -> bool {
    matches!(
        item,
        "minecraft:bow"
            | "minecraft:crossbow"
            | "minecraft:trident"
            | "minecraft:snowball"
            | "minecraft:egg"
            | "minecraft:ender_pearl"
            | "minecraft:experience_bottle"
            | "minecraft:splash_potion"
            | "minecraft:lingering_potion"
            | "minecraft:wind_charge"
            | "minecraft:fishing_rod"
    )
}
/// Equal physical drags turn equally on either axis, including portrait screens.
pub fn camera_delta(delta: [f32; 2], width: u32, height: u32, s: &Settings) -> [f32; 2] {
    let scale = width.max(height).max(1) as f32;
    [
        delta[0] * width as f32 / scale * 180.0 * s.sensitivity * s.sensitivity_horizontal,
        delta[1] * height as f32 / scale * 180.0 * s.sensitivity * s.sensitivity_vertical,
    ]
}

/// In tap modes the crosshair is absent, so use Java's native hotbar gauge.
pub fn attack_indicator(setting: u8, crosshair: bool) -> u8 {
    if setting == 1 && !crosshair {
        2
    } else {
        setting
    }
}

/// A finger ray through the perspective projection, without rotating the view.
pub fn screen_direction(
    forward: [f64; 3],
    up: [f64; 3],
    point: [f32; 2],
    aspect: f64,
    fov: f64,
) -> [f64; 3] {
    let right = [
        forward[1] * up[2] - forward[2] * up[1],
        forward[2] * up[0] - forward[0] * up[2],
        forward[0] * up[1] - forward[1] * up[0],
    ];
    let t = (fov.to_radians() / 2.0).tan();
    let x = (f64::from(point[0]) * 2.0 - 1.0) * aspect * t;
    let y = (1.0 - f64::from(point[1]) * 2.0) * t;
    let ray = std::array::from_fn::<_, 3, _>(|i| forward[i] + right[i] * x + up[i] * y);
    let length = ray.iter().map(|v| v * v).sum::<f64>().sqrt();
    ray.map(|v| v / length)
}

/// Multiplayer player-list button, uses the same normalized top row as other controls.
pub fn players_rect(width: f32, height: f32) -> [f32; 4] {
    let size = height * 0.075;
    [width * 0.62 - size / 2.0, height * 0.055 - size / 2.0, size, size]
}
pub fn players_at(width: f32, height: f32, point: [f32; 2]) -> bool {
    layout::contains(players_rect(width, height), point)
}
pub fn draw_players(b: &mut HudBatch, a: &HudAtlas, pressed: bool) {
    let rect = players_rect(b.width, b.height);
    text_button(b, a, rect, "Tab", pressed, rgba(255,255,255,255), true);
    let [x,y,w,h] = rect;
    for (p,q) in [([x,y],[x+w,y]),([x+w,y],[x+w,y+h]),([x+w,y+h],[x,y+h]),([x,y+h],[x,y])] { line(b,a,p,q,rgba(188,100,255,230)); }
}

/// The extra editor button shares the HUD's GUI-pixel coordinates with hotbar hit testing.
pub fn more_rect(width: f32, height: f32, avoid_left: bool) -> [f32; 4] {
    [width / 2.0 - 117.0, height - if avoid_left { 48.0 } else { 23.0 }, 22.0, 22.0]
}
pub fn more_at(width: f32, height: f32, point: [f32; 2], avoid_left: bool) -> bool {
    layout::contains(more_rect(width, height, avoid_left), point)
}
/// Compact actual player status; original vanilla survival bars remain intact.
pub fn draw_player_card(b: &mut HudBatch, a: &HudAtlas, name: &str, skin: Option<&str>, health: f32, max_health: f32, armor: i32) {
    let x = 40.0;
    let width = (b.width / 2.0 - x - 14.0).clamp(0.0, 120.0);
    if width < 75.0 { return; }
    let y = 3.0;
    b.fill(a, x, y, width, 30.0, rgba(12, 9, 25, 218));
    line(b, a, [x, y + 30.0], [x + width, y + 30.0], rgba(170, 95, 235, 200));
    let face = if width >= 100.0 {
        skin.and_then(|s| a.sprites.get(s)).or_else(|| a.sprites.get("entity/player/wide/steve")).copied()
    } else { None };
    let info_x = if face.is_some() { x + 25.0 } else { x + 3.0 };
    if let Some(r) = face {
        // PlayerFaceExtractor: 8x8 face at (8,8), then the hat at (40,8).
        for u in [8.0, 40.0] {
            let mut part = r;
            part.u0 = r.u0 + (r.u1-r.u0) * u / 64.0;
            part.u1 = r.u0 + (r.u1-r.u0) * (u+8.0) / 64.0;
            part.v0 = r.v0 + (r.v1-r.v0) * 8.0 / 64.0;
            part.v1 = r.v0 + (r.v1-r.v0) * 16.0 / 64.0;
            b.rect_uv(x + 3.0, y + 5.0, 19.0, 19.0, part, u32::MAX);
        }
    }
    let available = (x + width - info_x - 3.0).max(1.0);
    let mut label = name.to_owned();
    while HudBatch::text_width(a, &label) > available && !label.is_empty() { label.pop(); }
    b.text(a, &label, info_x, y + 2.0, rgba(255, 255, 255, 255), false);
    let step = (available / 10.0).min(7.0);
    for i in 0..10 {
        let icon_x = info_x + i as f32 * step;
        let remaining = health - i as f32 * 2.0;
        if (i as f32) * 2.0 < max_health.max(0.0) {
            b.sprite_scaled(a, "hud/heart/container", icon_x, y + 11.0, step, 7.0, u32::MAX);
            if remaining > 0.0 {
                b.sprite_scaled(a, if remaining >= 2.0 { "hud/heart/full" } else { "hud/heart/half" }, icon_x, y + 11.0, step, 7.0, u32::MAX);
            }
        }
        let remaining_armor = armor - i * 2;
        b.sprite_scaled(a, if remaining_armor >= 2 { "hud/armor_full" } else if remaining_armor == 1 { "hud/armor_half" } else { "hud/armor_empty" },
            icon_x, y + 20.0, step, 7.0, u32::MAX);
    }
    // Original survival bars still handle extra health, status effects and absorption.
}

/// Overlay only the slot borders: item sprites and the engine's selected slot
/// remain authoritative, and the geometry matches `hotbar_slot`.
pub fn draw_hotbar_neon(b: &mut HudBatch, a: &HudAtlas, selected: usize) {
    let start = b.width / 2.0 - 90.0;
    let y = b.height - 22.0;
    for slot in 0..9 {
        let x = start + slot as f32 * 20.0;
        let color = if slot == selected { rgba(226, 174, 255, 245) } else { rgba(166, 103, 224, 175) };
        for (p, q) in [([x, y], [x+20.0, y]), ([x+20.0, y], [x+20.0, y+20.0]),
            ([x+20.0, y+20.0], [x, y+20.0]), ([x, y+20.0], [x, y])] { line(b, a, p, q, color); }
    }
}
pub fn draw_more(b: &mut HudBatch, a: &HudAtlas, avoid_left: bool) {
    let [x, y, w, h] = more_rect(b.width, b.height, avoid_left);
    b.fill(a, x, y, w, h, rgba(15, 8, 30, 210));
    for (p, q) in [([x, y], [x+w, y]), ([x+w, y], [x+w, y+h]),
        ([x+w, y+h], [x, y+h]), ([x, y+h], [x, y])] {
        line(b, a, p, q, rgba(188, 100, 255, 235));
    }
    fitted_sprite(b, a, "ui/more-dots", [x + 4.0, y + 4.0, w - 8.0, h - 8.0], rgba(255,255,255,255));
}

pub fn hotbar_slot(width: f32, height: f32, point: [f32; 2]) -> Option<usize> {
    let x = point[0] - (width / 2.0 - 90.0);
    (point[1] >= height - 22.0 && point[1] < height && (0.0..180.0).contains(&x))
        .then(|| (x / 20.0) as usize)
}
use crate::craftix_controls::{
    editor::{Editor, Layouts},
    input::{Event, Router},
    layout::{self as hud, Action, Behavior, Icon, Id, Layout},
    paint::{self, Animator},
    ui::{Act, EditorUi, Tab, TextTarget},
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// The replacement control file, next to the legacy `horizon-touch.json`.
pub const FILE: &str = "craftix-controls.json";
const LEGACY_FILE: &str = "horizon-touch.json";

/// Layout files the Android Documents provider can copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutRequest {
    Import,
    Export,
    SaveSlot(u8),
    LoadSlot(u8),
}

/// Run an editor mutation, reporting its error in the panel instead of failing.
macro_rules! apply {
    ($session:ident, $call:expr) => {{
        let outcome = $call;
        if let Err(error) = outcome {
            $session.ui.status(format!("{error}"));
        }
    }};
}

#[derive(Clone, Debug)]
struct Look {
    last: [f32; 2],
    since: Instant,
    dragged: bool,
    committed: bool,
    held_use: bool,
}

/// One open editor session: its transactional data and its on-screen state.
pub struct Session {
    pub ui: EditorUi,
    data: Editor,
    /// The host IME was already asked for this field.
    request_sent: bool,
    request: Option<LayoutRequest>,
}

/// The live replacement control system. Everything the game reads from the HUD
/// goes through here; the pure model lives in `craftix_controls`.
pub struct TouchControls {
    /// Legacy scalar preferences (mode, sensitivity, margins) from options.
    pub settings: Settings,
    pub force_crosshair: bool,
    pub projectile_crosshair: bool,
    pub shield_available: bool,
    /// Physical keypad profiles keep utility controls without movement.
    pub auxiliary_only: bool,
    pub inventory_mode: bool,
    pub editor: Option<Session>,
    pub aim: Option<[f32; 2]>,
    pub chat_requested: bool,
    pub settings_requested: bool,
    pub debug_requested: bool,
    pub perspective_requested: bool,
    pub clicked: bool,
    pub context_use: bool,
    pub vertical_movement: bool,
    pub swimming: bool,
    pub mounted: bool,
    pub sneaking: bool,
    pub hold_uses_item: bool,
    pub holding_use: bool,
    pub holding: bool,
    pub break_progress: f32,
    layouts: Layouts,
    router: Router,
    animator: Animator,
    gestures: BTreeMap<u64, Look>,
    /// Engine tokens currently down, for the host's inventory/player queries.
    tokens: BTreeSet<String>,
    events: Vec<(String, bool)>,
    pending: PadState,
    tap_aim: Option<[f32; 2]>,
    pending_tap: Option<bool>,
    last_marker: Option<([f32; 2], Instant)>,
    look_delta: [f32; 2],
    sprint_latched: bool,
    ignored: BTreeSet<u64>,
    more_requested: bool,
    opened: bool,
    aspect: f32,
    viewport: Option<[u32; 2]>,
    frame: Instant,
    started: Instant,
    /// Where the named layouts are persisted.
    path: PathBuf,
    pub status: Option<String>,
}

impl Default for TouchControls {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            force_crosshair: false,
            projectile_crosshair: false,
            shield_available: false,
            auxiliary_only: false,
            inventory_mode: false,
            editor: None,
            aim: None,
            chat_requested: false,
            settings_requested: false,
            debug_requested: false,
            perspective_requested: false,
            clicked: false,
            context_use: false,
            vertical_movement: false,
            swimming: false,
            mounted: false,
            sneaking: false,
            hold_uses_item: false,
            holding_use: false,
            holding: false,
            break_progress: 0.0,
            layouts: Layouts::default(),
            router: Router::default(),
            animator: Animator::default(),
            gestures: BTreeMap::new(),
            tokens: BTreeSet::new(),
            events: Vec::new(),
            pending: PadState::default(),
            tap_aim: None,
            pending_tap: None,
            last_marker: None,
            look_delta: [0.0; 2],
            sprint_latched: false,
            ignored: BTreeSet::new(),
            more_requested: false,
            opened: false,
            aspect: 0.0,
            viewport: None,
            frame: Instant::now(),
            started: Instant::now(),
            path: PathBuf::from(FILE),
            status: None,
        }
    }
}

impl TouchControls {
    fn aspect(&self) -> f32 {
        if self.aspect > 0.0 {
            self.aspect
        } else {
            16.0 / 9.0
        }
    }
    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
    /// The active layout with the legacy global preferences applied.
    fn layout_now(&self) -> Layout {
        let mut layout = self.layouts.current().clone();
        let scale = if self.settings.button_scale.is_finite() {
            self.settings.button_scale.clamp(0.5, 2.0)
        } else {
            1.0
        };
        if (scale - 1.0).abs() > 0.001 {
            for button in &mut layout.buttons {
                button.circle.radius = (button.circle.radius * scale).clamp(0.015, 0.24);
            }
        }
        if self.settings.left_handed {
            layout.mirror();
        }
        // Tall surfaces (a portrait launcher window) leave little width for the
        // same fractions, so shrink the circles instead of letting them touch.
        let squeeze = (self.aspect() / 1.25).clamp(0.55, 1.0);
        if squeeze < 0.999 {
            let narrow = self.aspect();
            for button in &mut layout.buttons {
                button.circle.radius = (button.circle.radius * squeeze).clamp(0.015, 0.24);
                let half = button.circle.radius / narrow.max(0.1);
                button.circle.center[0] = button.circle.center[0].clamp(half, 1.0 - half);
                button.circle.center[1] = button.circle.center[1].clamp(button.circle.radius, 1.0 - button.circle.radius);
            }
        }
        layout
    }
    fn surface(&self) -> [f32; 2] {
        let batch = HudBatch::new(self.viewport.unwrap_or([1920, 1080]));
        [batch.width.max(1.0), batch.height.max(1.0)]
    }
    pub fn crosshair(&self) -> bool {
        self.force_crosshair || self.settings.crosshair()
    }
    /// Load the named layouts once per profile, migrating 0.8.x data if present.
    fn load(&mut self) {
        if self.opened {
            return;
        }
        self.opened = true;
        let existed = self.path.exists();
        let mut message = None;
        let mut layouts = if existed {
            let (layouts, problem) = Layouts::load(&self.path);
            message = problem;
            layouts
        } else if let Some(migrated) = migrate_legacy() {
            if let Err(error) = migrated.save(&self.path) {
                message = Some(format!("Could not save migrated controls: {error}"));
            }
            migrated
        } else {
            let layouts = Layouts::default();
            if let Err(error) = layouts.save(&self.path) {
                message = Some(format!("Controls will not persist: {error}"));
            }
            layouts
        };
        if !layouts.valid() {
            layouts = Layouts::default();
            message = Some("Control layouts were invalid and were reset".into());
        }
        self.layouts = layouts;
        self.status = message;
    }
    pub fn viewport(&mut self, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        if self.viewport.is_some_and(|previous| previous != size) {
            // Existing touch coordinates belong to the old drawable surface.
            self.clear();
        }
        self.viewport = Some(size);
        self.aspect = size[0] as f32 / size[1] as f32;
        self.load();
    }
    pub fn set_inventory_mode(&mut self, inventory: bool) {
        if self.inventory_mode != inventory {
            self.inventory_mode = inventory;
            self.router.set_inventory(inventory);
            self.tick_events();
        }
    }
    /// Pixel rectangle of every drawable button, for host hotspots.
    pub fn button_at_point(&self, point: [f32; 2]) -> Option<Id> {
        self.layout_now()
            .hit_in(point, self.aspect(), self.inventory_mode)
            .map(|button| button.id)
    }
    pub fn control_at(&self, point: [f32; 2]) -> Option<Control> {
        self.button_at_point(point).and_then(control_of)
    }
    pub fn control_at_point(&self, point: [f32; 2]) -> bool {
        self.button_at_point(point).is_some()
    }
    pub fn custom_at(&self, point: [f32; 2]) -> Option<usize> {
        match self.button_at_point(point)? {
            Id::Custom(index) => Some(index as usize),
            _ => None,
        }
    }
    /// The top bar and More: touches the host treats as menu gestures.
    pub fn auxiliary_at(&self, point: [f32; 2]) -> bool {
        matches!(
            self.button_at_point(point),
            Some(Id::Chat | Id::Pause | Id::Settings | Id::Perspective | Id::More)
        ) || self.custom_at(point).is_some()
    }
    pub fn owns_pointer(&self, id: u64) -> bool {
        self.router.owns(id) || self.ignored.contains(&id)
    }
    pub fn ignore(&mut self, id: u64) {
        self.ignored.insert(id);
        self.router.ignore(id);
    }
    pub fn tap(&mut self, button: Button) {
        self.pending.set(button, true);
    }
    /// Inventory overlays keep key buttons but never apply world gestures.
    pub fn clear_world_gesture(&mut self) {
        self.gestures.clear();
        self.sprint_latched = false;
        self.pending = PadState::default();
        self.aim = None;
        self.tap_aim = None;
        self.pending_tap = None;
        self.last_marker = None;
        self.holding = false;
        self.holding_use = false;
    }
    pub fn shield_held(&self) -> bool {
        // The replacement set raises the shield through sneak, as vanilla does.
        self.router.active(&Action::Sneak)
    }
    pub fn pointer(&self) -> Option<[f32; 2]> {
        if self.crosshair() {
            return None;
        }
        self.router.look_pointer().or_else(|| {
            self.last_marker
                .filter(|(_, at)| at.elapsed() < Duration::from_millis(180))
                .map(|(point, _)| point)
        })
    }
    /// Coordinates are fractions of the drawable surface. Returns a look delta.
    pub fn update(&mut self, id: u64, point: [f32; 2], started: bool, ended: bool) -> [f32; 2] {
        if !self.ignored.is_empty() {
            if self.ignored.contains(&id) {
                if ended && self.ignored.remove(&id) {
                    self.router.end(id);
                }
                return [0.0; 2];
            }
            if ended {
                self.ignored.remove(&id);
            }
        }
        let layout = self.layout_now();
        let aspect = self.aspect();
        if started {
            let mapped = self.inventory_mode
                && self
                    .inventory_token(point)
                    .is_some_and(|token| self.router.begin_mapped(id, Action::Key(token.into()), point));
            if !mapped {
                let now = self.now_ms();
                self.router.begin(&layout, id, point, aspect, now);
            }
        } else if !ended {
            self.router.moved(&layout, id, point, aspect);
            if let Some(look) = self.gestures.get_mut(&id) {
                look.last = point;
            }
        }
        if ended {
            self.router.end(id);
        }
        self.tick_events();
        std::mem::take(&mut self.look_delta)
    }
    fn inventory_token(&self, point: [f32; 2]) -> Option<&'static str> {
        match self.button_at_point(point)? {
            Id::Sneak => Some("key.keyboard.left.shift"),
            Id::Attack => Some("action:key.attack"),
            Id::Use => Some("action:key.use"),
            _ => None,
        }
    }
    /// Apply router edges to engine tokens and host requests.
    fn tick_events(&mut self) {
        let now = Instant::now();
        for event in self.router.drain() {
            match event {
                Event::Key(action, down) => {
                    if let Some(token) = action.token() {
                        let token = token.to_owned();
                        if down {
                            self.tokens.insert(token.clone());
                        } else {
                            self.tokens.remove(&token);
                        }
                        self.events.push((token, down));
                        continue;
                    }
                    if down {
                        match action {
                            Action::Chat => {
                                self.chat_requested = true;
                                self.clicked = true;
                            }
                            Action::Settings => {
                                self.settings_requested = true;
                                self.clicked = true;
                            }
                            Action::Perspective => {
                                self.perspective_requested = true;
                                self.clicked = true;
                            }
                            Action::More => {
                                self.more_requested = true;
                                self.clicked = true;
                            }
                            Action::Key(_) => {}
                            _ => {
                                self.clicked = true;
                            }
                        }
                    }
                }
                Event::Look(delta) => {
                    self.look_delta[0] += delta[0];
                    self.look_delta[1] += delta[1];
                }
                Event::LookBegin(pointer, start) => {
                    self.gestures.insert(
                        pointer,
                        Look {
                            last: start,
                            since: now,
                            dragged: false,
                            committed: false,
                            held_use: self.hold_uses_item,
                        },
                    );
                }
                Event::LookEnd { pointer, start, dragged } => {
                    let quick = self
                        .gestures
                        .remove(&pointer)
                        .is_some_and(|look| !look.dragged && look.since.elapsed() < HOLD_DELAY);
                    if quick && (!self.crosshair() || self.settings.crosshair_gestures) {
                        let aim = if self.crosshair() || self.projectile_crosshair {
                            [0.5, 0.5]
                        } else {
                            start
                        };
                        self.tap_aim = Some(aim);
                        self.last_marker = Some((start, now));
                        self.pending_tap = Some(self.hold_uses_item);
                    }
                    if !dragged {
                        self.last_marker = Some((start, now));
                    }
                }
                Event::Move(_) => {}
            }
        }
    }
    /// Held buttons, as the engine's pad mapping sees them.
    pub fn state(&self) -> PadState {
        let mut state = PadState::default();
        if self.settings.sprint_button
            && (self.router.active(&Action::Sprint) || self.sprint_latched)
        {
            state.set(Button::LeftStick, true);
        }
        if self.router.active(&Action::Jump) {
            state.set(Button::South, true);
        }
        if self.router.active(&Action::Sneak) {
            state.set(Button::RightStick, true);
        }
        if self.router.active(&Action::Attack) {
            state.triggers[1] = 1.0;
        }
        if self.router.active(&Action::Use) {
            state.triggers[0] = 1.0;
        }
        if self.router.active(&Action::Pause) {
            state.set(Button::Start, true);
        }
        if self.router.active(&Action::Inventory) {
            state.set(Button::North, true);
        }
        let mut movement = self.router.movement();
        if self.settings.mode == Mode::Dpad {
            movement = movement.map(|v| if v.abs() < 1.0 / 3.0 { 0.0 } else { v.signum() });
        } else if self.settings.sprint_with_joystick && self.router.reach() > 1.0 {
            state.set(Button::LeftStick, true);
        }
        let length = movement[0].hypot(movement[1]).max(1.0);
        state.left = movement.map(|v| v / length);
        state
    }
    pub fn poll_state(&mut self) -> PadState {
        let elapsed = self.frame.elapsed().as_secs_f32();
        self.frame = Instant::now();
        let layout = self.layout_now();
        self.animator.advance(&layout, &self.router, elapsed);
        let mut state = self.state().merge(&std::mem::take(&mut self.pending));
        self.aim = self.tap_aim.take();
        self.holding_use = false;
        self.holding = false;
        let crosshair = self.crosshair();
        if !crosshair || self.settings.crosshair_gestures {
            let now = Instant::now();
            let mut aim = None;
            let mut committed = None;
            for look in self.gestures.values_mut() {
                if !look.dragged && now.duration_since(look.since) >= HOLD_DELAY {
                    look.committed = true;
                }
                if lookup_applies(look) {
                    aim = Some(if crosshair || self.projectile_crosshair {
                        [0.5, 0.5]
                    } else {
                        look.last
                    });
                    if look.committed {
                        committed = Some(look.held_use);
                        break;
                    }
                }
            }
            if aim.is_some() {
                self.aim = aim;
            }
            if let Some(held_use) = committed {
                self.holding = true;
                self.holding_use = held_use;
                state.triggers[usize::from(!held_use)] = 1.0;
            }
        }
        let now = self.now_ms();
        self.router.tick(now);
        self.tick_events();
        if std::mem::take(&mut self.more_requested) {
            self.open_custom_manager();
        }
        state
    }
    /// Keep a contextual hold as use after feeding consumed its last item.
    pub fn latch_use(&mut self) {
        for look in self.gestures.values_mut() {
            if look.committed {
                look.held_use = true;
            }
        }
        self.holding_use = true;
    }
    pub fn clear(&mut self) {
        self.router.clear();
        self.tick_events();
        self.gestures.clear();
        self.ignored.clear();
        self.sprint_latched = false;
        self.pending = PadState::default();
        self.aim = None;
        self.tap_aim = None;
        self.pending_tap = None;
        self.last_marker = None;
        self.look_delta = [0.0; 2];
        self.holding = false;
        self.holding_use = false;
        self.chat_requested = false;
        self.settings_requested = false;
        self.debug_requested = false;
        self.perspective_requested = false;
        self.more_requested = false;
    }
    pub fn take_tap(&mut self) -> Option<bool> {
        self.pending_tap.take()
    }
    pub fn key_events(&mut self) -> Vec<(String, bool)> {
        std::mem::take(&mut self.events)
    }
    pub fn held_custom_tokens(&self) -> impl Iterator<Item = &str> {
        self.tokens.iter().map(String::as_str)
    }
    pub fn advance_custom_taps(&mut self) {
        let now = self.now_ms();
        self.router.tick(now);
        self.tick_events();
    }
    pub fn begin_mapped_touch(&mut self, id: u64, token: &str, point: [f32; 2]) -> bool {
        if !hud::valid_token(token) || self.router.owns(id) {
            return false;
        }
        let started = self.router.begin_mapped(id, Action::Key(token.to_owned()), point);
        if started {
            self.tick_events();
        }
        started
    }
    /// Inventory overlays turn the movement buttons into their real key edges.
    pub fn inventory_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if self.router.owns(id) {
            if ended {
                self.router.end(id);
            } else if !started {
                let layout = self.layout_now();
                self.router.moved(&layout, id, point, self.aspect());
            }
            self.tick_events();
            return true;
        }
        if started
            && let Some(token) = self.inventory_token(point)
        {
            self.tick_events();
            return self.router.begin_mapped(id, Action::Key(token.into()), point);
        }
        false
    }
    fn auxiliary_id(id: Id) -> bool {
        matches!(
            id,
            Id::Chat | Id::Pause | Id::Settings | Id::Perspective | Id::More
        )
    }
    pub fn draw(&self, b: &mut HudBatch, a: &HudAtlas) {
        let layout = self.layout_now();
        let inventory = self.inventory_mode;
        let auxiliary_only = self.auxiliary_only;
        paint::paint_filtered(b, a, &layout, &self.animator, &|button| {
            button.visibility.permits(inventory) && (!auxiliary_only || Self::auxiliary_id(button.id))
        });
        paint::paint_break_progress(b, a, &layout, self.break_progress);
        if let Some((point, at)) = self.last_marker {
            let age = at.elapsed().as_secs_f32();
            if age < 0.35 {
                paint::paint_tap_marker(b, a, point, 1.0 - age / 0.35);
            }
        }
        if let Some(session) = &self.editor {
            paint::paint_selection(b, a, &layout, session.ui.selected);
            session.ui.draw(b, a, &session.data);
        }
    }
    // --- editor -----------------------------------------------------------
    pub fn open_editor(&mut self) {
        self.clear();
        match Editor::new(self.layouts.clone()) {
            Ok(data) => {
                self.editor = Some(Session {
                    ui: EditorUi::default(),
                    data,
                    request_sent: false,
                    request: None,
                });
                self.status = None;
            }
            Err(error) => self.status = Some(format!("Editor unavailable: {error}")),
        }
    }
    pub fn open_custom_manager(&mut self) {
        self.open_editor();
        if let Some(session) = &mut self.editor {
            session.ui.tab = Tab::Buttons;
            session.ui.selected = session
                .data
                .buttons()
                .iter()
                .rev()
                .find(|button| button.id.is_custom())
                .map(|button| button.id)
                .or(Some(Id::Attack));
        }
    }
    pub fn take_layout_request(&mut self) -> Option<LayoutRequest> {
        self.editor.as_mut()?.request.take()
    }
    pub fn layout_status(&mut self, message: String) {
        match &mut self.editor {
            Some(session) => session.ui.status(message),
            None => self.status = Some(message),
        }
    }
    /// The host displays its real text input when a name field is tapped.
    pub fn take_custom_name_request(&mut self) -> Option<String> {
        let session = self.editor.as_mut()?;
        if session.request_sent {
            return None;
        }
        let initial = match session.ui.text? {
            TextTarget::Button(id) => session.data.label_of(id)?.to_owned(),
            TextTarget::Layout => session.data.staged.current().name.clone(),
        };
        session.request_sent = true;
        Some(initial)
    }
    pub fn custom_name_editing(&self) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|session| session.ui.text.is_some())
    }
    pub fn custom_name_text(&mut self, text: &str) {
        let Some(session) = self.editor.as_mut() else {
            return;
        };
        match session.ui.text {
            Some(TextTarget::Button(id)) => {
                let label = format!("{}{}", session.data.label_of(id).unwrap_or(""), text);
                if let Err(error) = session.data.rename_button(id, &label) {
                    session.ui.status(format!("Name rejected: {error}"));
                }
            }
            Some(TextTarget::Layout) => {
                let name = format!("{}{}", session.data.staged.current().name, text);
                if let Err(error) = session.data.rename(&name) {
                    session.ui.status(format!("Name rejected: {error}"));
                }
            }
            None => {}
        }
    }
    pub fn custom_name_backspace(&mut self) {
        let Some(session) = self.editor.as_mut() else {
            return;
        };
        match session.ui.text {
            Some(TextTarget::Button(id)) => {
                let mut label = session.data.label_of(id).unwrap_or("").to_owned();
                label.pop();
                let _ = session.data.rename_button(id, &label);
            }
            Some(TextTarget::Layout) => {
                let mut name = session.data.staged.current().name.clone();
                if name.chars().count() > 1 {
                    name.pop();
                    let _ = session.data.rename(&name);
                }
            }
            None => {}
        }
    }
    pub fn finish_custom_name(&mut self) {
        if let Some(session) = self.editor.as_mut() {
            session.ui.text = None;
            session.request_sent = false;
        }
    }
    pub fn cancel_editor(&mut self) {
        if let Some(session) = self.editor.take() {
            self.layouts = session.data.cancel_copy();
        }
        self.clear();
    }
    /// The editor owns every finger: editing cannot move, mine or open menus.
    /// Returns true when Save or Cancel closed the editor.
    pub fn edit_touch(&mut self, id: u64, p: [f32; 2], started: bool, ended: bool) -> bool {
        let Some(mut session) = self.editor.take() else {
            return false;
        };
        self.clicked |= started;
        if session.ui.text.is_some() {
            self.editor = Some(session);
            return false;
        }
        let surface = self.surface();
        let aspect = surface[0] / surface[1].max(1.0);
        let mut closed = false;
        if started {
            if session.ui.tab == Tab::Position && session.data.begin_drag(id, p, aspect) {
                session.ui.status("Drag to move. Save keeps it.");
            } else if let Some(act) = session.ui.hit(&session.data, surface[0], surface[1], p) {
                closed = self.apply_editor(&mut session, act);
            }
        } else if !ended {
            session.data.move_drag(id, p, aspect);
        }
        if ended {
            session.data.end_drag(id);
        }
        if !closed {
            self.editor = Some(session);
        }
        closed
    }
    /// Apply one editor command. Returns true when the session was closed.
    fn apply_editor(&mut self, session: &mut Session, act: Act) -> bool {
        let selected = session.ui.selected;
        match act {
            Act::Tab(tab) => {
                session.ui.tab = tab;
                session.ui.picker = false;
                session.ui.capturing = false;
                session.ui.confirm_delete = false;
            }
            Act::Select(id) => {
                session.ui.selected = Some(id);
                session.ui.confirm_delete = false;
            }
            Act::AddKey => {
                session.ui.capturing = true;
                session.ui.message = None;
            }
            Act::Picker => {
                if selected.is_some() {
                    session.ui.picker = true;
                    session.ui.picker_page = 0;
                    session.ui.picker_raw = false;
                }
            }
            Act::Pick(index) => {
                let choices = EditorUi::choices(session.ui.picker_raw);
                if let (Some(id), Some((token, label))) = (selected, choices.get(index).cloned()) {
                    match session.data.set_binding(id, &token) {
                        Ok(()) => {
                            session.ui.status(format!("Bound to {label}"));
                            session.ui.picker = false;
                        }
                        Err(error) => session.ui.status(format!("Binding rejected: {error}")),
                    }
                }
            }
            Act::PickerPage(forward) => {
                let pages = EditorUi::page_count(session.ui.picker_raw);
                session.ui.picker_page = if forward {
                    (session.ui.picker_page + 1).min(pages - 1)
                } else {
                    session.ui.picker_page.saturating_sub(1)
                };
            }
            Act::PickerRaw => {
                session.ui.picker_raw = !session.ui.picker_raw;
                session.ui.picker_page = 0;
            }
            Act::ClosePicker => session.ui.picker = false,
            Act::Rename => {
                if let Some(id) = selected {
                    session.ui.text = Some(TextTarget::Button(id));
                    session.request_sent = false;
                }
            }
            Act::RenameLayout => {
                session.ui.text = Some(TextTarget::Layout);
                session.request_sent = false;
            }
            Act::CycleIcon => {
                if let Some(id) = selected {
                    apply!(session, session.data.next_icon(id));
                }
            }
            Act::CycleBehavior => {
                if let Some(id) = selected {
                    apply!(session, session.data.cycle_behavior(id));
                }
            }
            Act::Size(bigger) => {
                if let Some(id) = selected {
                    let factor = if bigger { 1.12 } else { 1.0 / 1.12 };
                    apply!(session, session.data.scale(id, factor));
                }
            }
            Act::Opacity(bigger) => {
                if let Some(id) = selected {
                    let current = session
                        .data
                        .buttons()
                        .iter()
                        .find(|button| button.id == id)
                        .map(|button| button.opacity)
                        .unwrap_or(0.85);
                    let step = if bigger { 0.1 } else { -0.1 };
                    apply!(session, session.data.set_opacity(id, current + step));
                }
            }
            Act::ToggleEnabled => {
                if let Some(id) = selected {
                    apply!(session, session.data.toggle_enabled(id));
                }
            }
            Act::CycleVisibility => {
                if let Some(id) = selected {
                    apply!(session, session.data.cycle_visibility(id));
                }
            }
            Act::Front => {
                if let Some(id) = selected {
                    apply!(session, session.data.move_to_front(id));
                }
            }
            Act::DeleteButton => {
                if let Some(id) = selected {
                    if !session.ui.confirm_delete {
                        session.ui.confirm_delete = true;
                        session.ui.status("Tap Delete again to remove this button");
                    } else {
                        session.ui.confirm_delete = false;
                        match session.data.delete_button(id) {
                            Ok(()) => {
                                session.ui.selected = None;
                                session.ui.status("Button deleted");
                            }
                            Err(error) => session.ui.status(format!("{error}")),
                        }
                    }
                }
            }
            Act::Layout(index) => {
                match session.data.select(index) {
                    Ok(()) => session.ui.selected = None,
                    Err(error) => session.ui.status(format!("{error}")),
                }
                session.ui.confirm_delete = false;
            }
            Act::NewLayout => {
                let name = free_name(&session.data.staged.items, "Layout");
                apply!(session, session.data.create(&name));
                session.ui.selected = None;
            }
            Act::CopyLayout => {
                let name = free_name(&session.data.staged.items, "Copy");
                apply!(session, session.data.duplicate(&name));
                session.ui.selected = None;
            }
            Act::DeleteLayout => {
                if !session.ui.confirm_delete {
                    session.ui.confirm_delete = true;
                    session.ui.status("Tap Delete again to remove this layout");
                } else {
                    session.ui.confirm_delete = false;
                    apply!(session, session.data.delete());
                    session.ui.selected = None;
                }
            }
            Act::MakeDefault => {
                let active = session.data.staged.active;
                apply!(session, session.data.staged.set_default(active));
            }
            Act::Mirror => session.data.mirror(),
            Act::Factory => {
                session.data.factory_positions();
                session.ui.status("Factory positions restored");
            }
            Act::Export => session.request = Some(LayoutRequest::Export),
            Act::Import => session.request = Some(LayoutRequest::Import),
            Act::Dismiss => session.ui.message = None,
            Act::Save => {
                let result = session.data.commit_copy();
                return match result {
                    Ok(layouts) => {
                        let previous = std::mem::replace(&mut self.layouts, layouts);
                        if let Err(error) = self.persist() {
                            self.layouts = previous;
                            session.ui.status(format!("Save failed: {error}"));
                            return false;
                        }
                        self.router.clear();
                        self.tick_events();
                        self.status = None;
                        true
                    }
                    Err(error) => {
                        session.ui.status(format!("Cannot save: {error}"));
                        false
                    }
                };
            }
            Act::Cancel => {
                self.layouts = session.data.cancel_copy();
                self.router.clear();
                self.tick_events();
                return true;
            }
        }
        false
    }
    fn persist(&self) -> std::io::Result<()> {
        self.layouts.save(&self.path)
    }
    /// Write the named layouts where the Documents provider picks them up.
    pub fn export_layout(&self) -> std::io::Result<Vec<u8>> {
        self.layouts.to_json()
    }
    pub fn import_layout(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let layouts = Layouts::from_json(bytes)?;
        self.layouts = layouts;
        self.router.clear();
        self.tick_events();
        if let Some(session) = &mut self.editor {
            match Editor::new(self.layouts.clone()) {
                Ok(data) => {
                    let tab = session.ui.tab;
                    session.data = data;
                    session.ui = EditorUi { tab, ..EditorUi::default() };
                }
                Err(error) => session.ui.status(format!("{error}")),
            }
        } else {
            self.persist()?;
        }
        Ok(())
    }
    pub fn capture_custom_key(&mut self, name: &str) -> bool {
        if !hud::valid_token(name) {
            return false;
        }
        let Some(session) = self.editor.as_mut() else {
            return false;
        };
        if !session.ui.capturing {
            return false;
        }
        let label = Action::Key(name.to_owned()).label();
        match session.data.add_key(&label, name, Icon::Text) {
            Ok(id) => {
                session.ui.selected = Some(id);
                session.ui.capturing = false;
                session.ui.status(format!("Added {label}"));
                self.clicked = true;
                true
            }
            Err(error) => {
                session.ui.status(format!("Cannot add: {error}"));
                false
            }
        }
    }
}

fn lookup_applies(look: &Look) -> bool {
    !look.dragged || look.committed
}

fn free_name(existing: &[Layout], base: &str) -> String {
    for index in 2..100 {
        let name = format!("{base} {index}");
        if !existing
            .iter()
            .any(|layout| layout.name.trim().eq_ignore_ascii_case(&name))
        {
            return name;
        }
    }
    base.to_owned()
}

fn control_of(id: Id) -> Option<Control> {
    Some(match id {
        Id::Joystick => Control::Move,
        Id::Sprint => Control::Sprint,
        Id::Jump => Control::Jump,
        Id::Sneak => Control::Sneak,
        Id::Attack => Control::Attack,
        Id::Use => Control::Use,
        Id::Inventory => Control::Inventory,
        Id::Chat => Control::Chat,
        Id::Pause => Control::Pause,
        Id::Settings => Control::Settings,
        Id::Perspective => Control::Perspective,
        Id::Players | Id::More | Id::Custom(_) => return None,
    })
}

/// Carry a 0.8.x profile over once, keeping real keys and positions.
fn migrate_legacy() -> Option<Layouts> {
    let path = Path::new(LEGACY_FILE);
    if !path.exists() {
        return None;
    }
    let settings = Settings::load_from(path);
    let mut layout = Layout::factory("Migrated");
    let row = &settings.layouts[Mode::Crosshair.index()];
    for (id, control) in [
        (Id::Joystick, Control::Move),
        (Id::Sprint, Control::Sprint),
        (Id::Jump, Control::Jump),
        (Id::Sneak, Control::Sneak),
        (Id::Attack, Control::Attack),
        (Id::Use, Control::Use),
        (Id::Inventory, Control::Inventory),
        (Id::Chat, Control::Chat),
        (Id::Pause, Control::Pause),
        (Id::Settings, Control::Settings),
        (Id::Perspective, Control::Perspective),
    ] {
        let placement = row[control as usize];
        if let Some(button) = layout.button_mut(id) {
            button.circle.center = [
                placement.center[0].clamp(0.0, 1.0),
                placement.center[1].clamp(0.0, 1.0),
            ];
            button.circle.radius = (placement.size * 0.5).clamp(0.015, 0.24);
            button.opacity = placement.opacity.clamp(0.0, 1.0);
        }
    }
    for (index, legacy) in settings.custom_buttons.iter().enumerate() {
        if !hud::valid_token(&legacy.key) {
            continue;
        }
        let placement = legacy.layouts[Mode::Crosshair.index()];
        let mut button = hud::Button::new(
            Id::Custom(index as u32),
            Some(Action::Key(legacy.key.clone())),
            [
                placement.center[0].clamp(0.0, 1.0),
                placement.center[1].clamp(0.0, 1.0),
            ],
            (placement.size * 0.5).clamp(0.015, 0.24),
        );
        button.opacity = placement.opacity.clamp(0.0, 1.0);
        button.label = legacy.name.chars().filter(|c| !c.is_control()).take(32).collect();
        button.behavior = if legacy.single {
            Behavior::Single
        } else if legacy.toggle {
            Behavior::Toggle
        } else {
            Behavior::Hold
        };
        button.visibility = match legacy.visibility {
            CustomVisibility::Always => hud::Visibility::Always,
            CustomVisibility::Gameplay => hud::Visibility::Gameplay,
            CustomVisibility::Inventory => hud::Visibility::Inventory,
        };
        button.icon = match legacy.icon {
            CustomIcon::Text => Icon::Text,
            CustomIcon::Attack => Icon::Sword,
            CustomIcon::Use => Icon::Use,
            CustomIcon::Jump => Icon::Jump,
            CustomIcon::Sneak => Icon::Sneak,
            CustomIcon::Sprint => Icon::Sprint,
        };
        if button.valid() {
            layout.buttons.push(button);
        }
    }
    if !layout.valid() {
        layout = Layout::factory("Migrated");
    }
    Some(Layouts { schema: 1, active: 0, default: 0, items: vec![layout] })
}

/// Original optional resource-pack artwork. Missing states keep the default UI.
#[allow(clippy::too_many_arguments)]
pub fn theme_button(
    b: &mut HudBatch,
    a: &HudAtlas,
    name: &str,
    bounds: [f32; 4],
    pressed: bool,
    color: u32,
    frame: bool,
) -> bool {
    let (base, active) = match name {
        "chat" => (
            "gui/horizon/touch/chat_normal",
            "gui/horizon/touch/chat_pressed",
        ),
        "pause" => (
            "gui/horizon/touch/pause_normal",
            "gui/horizon/touch/pause_pressed",
        ),
        "settings" => (
            "gui/horizon/touch/settings_normal",
            "gui/horizon/touch/settings_pressed",
        ),
        "debug" => (
            "gui/horizon/touch/debug_normal",
            "gui/horizon/touch/debug_pressed",
        ),
        "perspective" => (
            "gui/horizon/touch/perspective_normal",
            "gui/horizon/touch/perspective_pressed",
        ),
        "button" => (
            "gui/horizon/touch/button_normal",
            "gui/horizon/touch/button_pressed",
        ),
        _ => return false,
    };
    let key = if pressed && a.sprites.contains_key(active) {
        active
    } else {
        base
    };
    let Some(rect) = a.sprites.get(key).copied() else {
        return false;
    };
    let [x, y, w, h] = bounds;
    if frame {
        crate::ui::nine_slice(b, a, key, x, y, w, h, color);
    } else {
        control_quad(b, a, x, y, w, h, rect, color);
    }
    true
}

/// Shared text controls: keep the raised frame at rest, visibly depress it while held.
fn text_button(
    b: &mut HudBatch,
    a: &HudAtlas,
    bounds: [f32; 4],
    label: &str,
    pressed: bool,
    color: u32,
    themed: bool,
) {
    let custom_art = themed && theme_button(b, a, "button", bounds, pressed, color, true);
    if !custom_art {
        let [r, g, blue, alpha] = color.to_le_bytes();
        let tint = if pressed {
            rgba(r / 2, g / 2, blue / 2, alpha)
        } else {
            color
        };
        menu_frame(b, a, bounds, tint);
    }
    let [x, y, w, h] = bounds;
    let offset = if pressed { (h / 18.0).min(2.0) } else { 0.0 };
    let width = HudBatch::text_width(a, label);
    let start = b.vertices.len();
    let label_color = if custom_art {
        rgba(25, 25, 25, color.to_le_bytes()[3])
    } else {
        color
    };
    b.text(
        a,
        label,
        x + (w - width) / 2.0,
        y + (h - 8.0) / 2.0 + offset,
        label_color,
        false,
    );
    let scale = ((w - 6.0).max(1.0) / width.max(1.0)).min(((h - 6.0).max(1.0) / 8.0).min(1.0));
    b.scale_since(start, x + w / 2.0, y + h / 2.0 + offset, scale, scale);
}

/// Reuse Chat's complete 18x18 border, including its bottommost pixel row.
fn menu_frame(b: &mut HudBatch, a: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    let border = (w / 18.0 * 2.0).min(h / 2.0);
    let xs = [200.0, 202.0, 216.0, 218.0];
    let ys = [82.0, 84.0, 96.0, 100.0];
    let dx = [x, x + border, x + w - border, x + w];
    let dy = [y, y + border, y + h - border * 2.0, y + h];
    for row in 0..3 {
        for col in 0..3 {
            let source = if row == 1 && col == 1 {
                [202.0, 84.0, 1.0, 1.0]
            } else {
                [
                    xs[col],
                    ys[row],
                    xs[col + 1] - xs[col],
                    ys[row + 1] - ys[row],
                ]
            };
            region(
                b,
                a,
                "gui",
                source,
                [
                    dx[col],
                    dy[row],
                    dx[col + 1] - dx[col],
                    dy[row + 1] - dy[row],
                ],
                color,
            );
        }
    }
}

fn nine(
    b: &mut HudBatch,
    a: &HudAtlas,
    name: &str,
    border: f32,
    [x, y, w, h]: [f32; 4],
    color: u32,
) {
    if let Some(r) = a.sprites.get(&format!("gui/bedrock/{name}")).copied() {
        let xs = [0.0, border, r.w as f32 - border, r.w as f32];
        let ys = [0.0, border, r.h as f32 - border, r.h as f32];
        let dx = [x, x + border, x + w - border, x + w];
        let dy = [y, y + border, y + h - border, y + h];
        for row in 0..3 {
            for col in 0..3 {
                let mut cell = r;
                cell.u0 = r.u0 + (r.u1 - r.u0) * xs[col] / r.w as f32;
                cell.u1 = r.u0 + (r.u1 - r.u0) * xs[col + 1] / r.w as f32;
                cell.v0 = r.v0 + (r.v1 - r.v0) * ys[row] / r.h as f32;
                cell.v1 = r.v0 + (r.v1 - r.v0) * ys[row + 1] / r.h as f32;
                control_quad(
                    b,
                    a,
                    dx[col],
                    dy[row],
                    dx[col + 1] - dx[col],
                    dy[row + 1] - dy[row],
                    cell,
                    color,
                );
            }
        }
    }
}

fn sprite(b: &mut HudBatch, a: &HudAtlas, name: &str, [x, y, w, h]: [f32; 4], color: u32) {
    fitted_sprite(b, a, name, [x, y, w, h], color);
}
fn mining_radius(radius: f32, progress: f32) -> f32 {
    if progress.is_finite() {
        (radius - 1.5).max(0.0) * progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
fn circle_points() -> &'static [[f32; 2]; 65] {
    static POINTS: std::sync::OnceLock<[[f32; 2]; 65]> = std::sync::OnceLock::new();
    POINTS.get_or_init(|| {
        std::array::from_fn(|i| {
            let angle = i as f32 / 64.0 * std::f32::consts::TAU;
            [angle.cos(), angle.sin()]
        })
    })
}
// One non-overlapping quad per segment, rather than many blended pixel squares.
fn ring(b: &mut HudBatch, a: &HudAtlas, center: [f32; 2], radius: f32, color: u32) {
    let Some(white) = a.sprites.get("horizon/white") else {
        return;
    };
    let uv = [[(white.u0 + white.u1) / 2.0, (white.v0 + white.v1) / 2.0]; 4];
    for segment in circle_points().windows(2) {
        let point = |p: [f32; 2], r: f32| [center[0] + p[0] * r, center[1] + p[1] * r];
        b.quad(
            [
                point(segment[0], radius + 0.7),
                point(segment[1], radius + 0.7),
                point(segment[1], (radius - 0.7).max(0.0)),
                point(segment[0], (radius - 0.7).max(0.0)),
            ],
            uv,
            color,
        );
    }
}
/// Glyphs such as the inventory ellipsis are deliberately wide and short.
/// Fit their original pixels inside the hit target without stretching them.
fn fitted_sprite(b: &mut HudBatch, a: &HudAtlas, name: &str, [x, y, w, h]: [f32; 4], color: u32) {
    let key = format!("gui/bedrock/{name}");
    if let Some(rect) = a.sprites.get(&key).copied() {
        let scale = (w / rect.w as f32).min(h / rect.h as f32);
        let (width, height) = (rect.w as f32 * scale, rect.h as f32 * scale);
        control_quad(
            b,
            a,
            x + (w - width) / 2.0,
            y + (h - height) / 2.0,
            width,
            height,
            rect,
            color,
        );
    }
}
fn region(
    b: &mut HudBatch,
    a: &HudAtlas,
    name: &str,
    [sx, sy, sw, sh]: [f32; 4],
    [x, y, w, h]: [f32; 4],
    color: u32,
) {
    if let Some(mut r) = a.sprites.get(&format!("gui/bedrock/{name}")).copied() {
        let (u, v) = (r.u0, r.v0);
        let (du, dv) = ((r.u1 - u) / r.w as f32, (r.v1 - v) / r.h as f32);
        r.u0 = u + sx * du;
        r.u1 = u + (sx + sw) * du;
        r.v0 = v + sy * dv;
        r.v1 = v + (sy + sh) * dv;
        control_quad(b, a, x, y, w, h, r, color);
    }
}

/// Atlas-wide clamp cannot protect an individual button from adjacent texels.
/// Keep UVs at texel centres and snap both screen edges to physical pixels.
#[allow(clippy::too_many_arguments)]
fn control_quad(
    b: &mut HudBatch,
    a: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    mut r: crate::hud::Rect,
    color: u32,
) {
    let half = [0.5 / a.width.max(1) as f32, 0.5 / a.height.max(1) as f32];
    r.u0 += half[0];
    r.u1 -= half[0];
    r.v0 += half[1];
    r.v1 -= half[1];
    let snap = |v: f32| (v * b.scale).round() / b.scale;
    let (left, top, right, bottom) = (snap(x), snap(y), snap(x + w), snap(y + h));
    b.rect_uv(left, top, right - left, bottom - top, r, color);
}

fn line(
    b: &mut crate::hud::HudBatch,
    a: &crate::hud::HudAtlas,
    p: [f32; 2],
    q: [f32; 2],
    color: u32,
) {
    // One quad per segment rather than one quad per pixel. Hotbar outlines,
    // editor glass and circular rings otherwise generate thousands of quads.
    let dx = q[0] - p[0];
    let dy = q[1] - p[1];
    let length = dx.hypot(dy);
    if length <= f32::EPSILON {
        b.fill(a, p[0] - 0.7, p[1] - 0.7, 1.4, 1.4, color);
        return;
    }
    let (nx, ny) = (-dy / length * 0.7, dx / length * 0.7);
    b.fill_quad(a, [
        [p[0] - nx, p[1] - ny], [q[0] - nx, q[1] - ny],
        [q[0] + nx, q[1] + ny], [p[0] + nx, p[1] + ny],
    ], color);
}
#[cfg(test)]
mod tests {
    use super::*;

    fn temp_controls(name: &str) -> TouchControls {
        let mut controls = TouchControls::default();
        let mut path = std::env::temp_dir();
        path.push(format!(
            "craftix-controls-test-{}-{name}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("craftix-tmp"));
        controls.path = path;
        controls
    }
    fn center(controls: &TouchControls, id: Id) -> [f32; 2] {
        controls
            .layouts
            .current()
            .button(id)
            .unwrap()
            .circle
            .center
    }
    fn widget_point(controls: &TouchControls, act: Act) -> [f32; 2] {
        let surface = controls.surface();
        let session = controls.editor.as_ref().unwrap();
        let widget = session
            .ui
            .widgets(&session.data, surface[0], surface[1])
            .into_iter()
            .rev()
            .find(|widget| widget.act == act)
            .unwrap();
        [
            (widget.rect[0] + widget.rect[2] / 2.0) / surface[0],
            (widget.rect[1] + widget.rect[3] / 2.0) / surface[1],
        ]
    }

    #[test]
    fn held_buttons_become_real_engine_buttons_and_release_on_lift() {
        let mut controls = TouchControls::default();
        let jump = center(&controls, Id::Jump);
        controls.update(1, jump, true, false);
        assert!(controls.poll_state().held(Button::South));
        controls.update(1, jump, false, true);
        assert!(!controls.poll_state().held(Button::South));
    }

    #[test]
    fn joystick_attack_and_jump_share_the_surface_without_losing_each_other() {
        let mut controls = TouchControls::default();
        let joystick = center(&controls, Id::Joystick);
        let attack = center(&controls, Id::Attack);
        let jump = center(&controls, Id::Jump);
        controls.update(1, joystick, true, false);
        controls.update(1, [joystick[0], joystick[1] - 0.12], false, false);
        controls.update(2, attack, true, false);
        controls.update(3, jump, true, false);
        let state = controls.poll_state();
        assert!(state.held(Button::South));
        assert_eq!(state.triggers[1], 1.0);
        assert!(state.left[1] > 0.0);
        controls.update(2, attack, false, true);
        let state = controls.poll_state();
        assert_eq!(state.triggers[1], 0.0);
        assert!(state.held(Button::South));
    }

    #[test]
    fn a_short_tap_aims_and_requests_interaction_without_turning_the_camera() {
        let mut controls = TouchControls::default();
        // Tap mode has no fixed crosshair, so a world touch can aim where it lands.
        controls.settings.mode = Mode::Tap;
        assert_eq!(controls.update(1, [0.5, 0.45], true, false), [0.0, 0.0]);
        controls.update(1, [0.5, 0.45], false, true);
        assert_eq!(controls.take_tap(), Some(false));
        let _ = controls.poll_state();
        let aim = controls.aim.expect("the tap should aim");
        assert!((aim[0] - 0.5).abs() < 1e-6 && (aim[1] - 0.45).abs() < 1e-6);
    }
    #[test]
    fn a_camera_drag_returns_a_delta_and_never_becomes_a_tap() {
        let mut controls = TouchControls::default();
        controls.update(1, [0.5, 0.5], true, false);
        let delta = controls.update(1, [0.6, 0.5], false, false);
        assert!(delta[0] > 0.0);
        controls.update(1, [0.6, 0.5], false, true);
        assert_eq!(controls.take_tap(), None);
    }

    #[test]
    fn two_fingers_on_one_key_emit_balanced_shared_edges() {
        let mut controls = TouchControls::default();
        let mut layout = controls.layouts.current().clone();
        layout.buttons.push(hud::Button::new(
            Id::Custom(0),
            Some(Action::Key("key.keyboard.m".into())),
            [0.55, 0.30],
            0.04,
        ));
        controls.layouts.items[0] = layout;
        controls.update(1, [0.55, 0.30], true, false);
        controls.update(2, [0.55, 0.30], true, false);
        assert_eq!(
            controls.key_events(),
            vec![("key.keyboard.m".to_string(), true)]
        );
        assert!(controls.held_custom_tokens().any(|token| token == "key.keyboard.m"));
        controls.update(1, [0.55, 0.30], false, true);
        assert!(controls.key_events().is_empty());
        controls.update(2, [0.55, 0.30], false, true);
        assert_eq!(
            controls.key_events(),
            vec![("key.keyboard.m".to_string(), false)]
        );
    }

    #[test]
    fn losing_focus_releases_every_held_key_and_button() {
        let mut controls = temp_controls("focus");
        controls.viewport(1920, 1080);
        let jump = center(&controls, Id::Jump);
        controls.update(1, jump, true, false);
        assert!(controls.poll_state().held(Button::South));
        controls.clear();
        assert!(!controls.poll_state().held(Button::South));
        assert!(controls.key_events().is_empty());
    }

    #[test]
    fn a_rotation_clears_held_touches_without_stranding_a_button() {
        let mut controls = temp_controls("rotate");
        controls.viewport(1920, 1080);
        let jump = center(&controls, Id::Jump);
        controls.update(1, jump, true, false);
        assert!(controls.poll_state().held(Button::South));
        controls.viewport(1080, 2400);
        assert!(!controls.poll_state().held(Button::South));
    }

    #[test]
    fn inventory_mode_sends_real_key_edges_instead_of_world_gestures() {
        let mut controls = TouchControls::default();
        controls.set_inventory_mode(true);
        let attack = center(&controls, Id::Attack);
        controls.update(1, attack, true, false);
        assert_eq!(
            controls.key_events(),
            vec![("action:key.attack".to_string(), true)]
        );
        assert_eq!(controls.take_tap(), None);
        controls.update(1, attack, false, true);
        assert_eq!(
            controls.key_events(),
            vec![("action:key.attack".to_string(), false)]
        );
    }

    #[test]
    fn the_top_bar_asks_the_host_for_chat_and_perspective() {
        let mut controls = TouchControls::default();
        let chat = center(&controls, Id::Chat);
        let perspective = center(&controls, Id::Perspective);
        controls.update(1, chat, true, true);
        assert!(controls.chat_requested);
        controls.update(2, perspective, true, true);
        assert!(controls.perspective_requested);
        let more = center(&controls, Id::More);
        controls.update(3, more, true, true);
        let _ = controls.poll_state();
        assert!(controls.editor.is_some(), "More opens the editor directly");
    }

    #[test]
    fn named_layouts_survive_a_restart_with_the_chosen_default() {
        let mut controls = temp_controls("persist");
        controls.viewport(1920, 1080);
        controls.open_editor();
        let mut session = controls.editor.take().unwrap();
        session.data.create("Second").unwrap();
        session.data.staged.set_default(1).unwrap();
        assert!(controls.apply_editor(&mut session, Act::Save));
        let mut restarted = TouchControls::default();
        restarted.path = controls.path.clone();
        restarted.viewport(1920, 1080);
        assert_eq!(restarted.layouts.items.len(), 2);
        assert_eq!(restarted.layouts.default, 1);
        assert_eq!(restarted.layouts.current().name, "Second");
        let _ = std::fs::remove_file(&controls.path);
    }

    #[test]
    fn cancel_restores_the_previous_layouts_without_touching_the_file() {
        let mut controls = temp_controls("cancel");
        controls.viewport(1920, 1080);
        let before = std::fs::read(&controls.path).unwrap();
        controls.open_editor();
        let mut session = controls.editor.take().unwrap();
        session.data.create("Scratch").unwrap();
        assert!(controls.apply_editor(&mut session, Act::Cancel));
        assert_eq!(controls.layouts.items.len(), 1);
        assert_eq!(std::fs::read(&controls.path).unwrap(), before);
        let _ = std::fs::remove_file(&controls.path);
    }

    #[test]
    fn tapping_save_in_the_panel_commits_and_closes_the_editor() {
        let mut controls = temp_controls("save-tap");
        controls.viewport(1920, 1080);
        controls.open_editor();
        let point = widget_point(&controls, Act::Save);
        assert!(controls.edit_touch(7, point, true, true));
        assert!(controls.editor.is_none());
        assert!(controls.layouts.valid());
        let _ = std::fs::remove_file(&controls.path);
    }

    #[test]
    fn the_position_tab_drags_buttons_and_save_keeps_the_new_place() {
        let mut controls = temp_controls("drag");
        controls.viewport(1920, 1080);
        controls.open_editor();
        controls.editor.as_mut().unwrap().ui.tab = Tab::Position;
        let start = center(&controls, Id::Jump);
        let moved = [start[0] - 0.08, start[1] - 0.06];
        assert!(!controls.edit_touch(3, start, true, false));
        assert!(!controls.edit_touch(3, moved, false, false));
        assert!(!controls.edit_touch(3, moved, false, true));
        let staged = controls
            .editor
            .as_ref()
            .unwrap()
            .data
            .buttons()
            .iter()
            .find(|button| button.id == Id::Jump)
            .unwrap()
            .circle
            .center;
        assert!((staged[0] - moved[0]).abs() < 1e-3, "{staged:?}");
        let mut session = controls.editor.take().unwrap();
        assert!(controls.apply_editor(&mut session, Act::Save));
        let saved = controls.layouts.current().button(Id::Jump).unwrap().circle.center;
        assert!((saved[0] - moved[0]).abs() < 1e-3);
        let _ = std::fs::remove_file(&controls.path);
    }

    #[test]
    fn capturing_a_physical_key_creates_a_working_button() {
        let mut controls = temp_controls("capture");
        controls.viewport(1920, 1080);
        controls.open_editor();
        assert!(!controls.capture_custom_key("key.keyboard.m"));
        let point = widget_point(&controls, Act::AddKey);
        assert!(!controls.edit_touch(1, point, true, true));
        assert!(!controls.capture_custom_key("key.keyboard.not-real"));
        assert!(controls.capture_custom_key("key.keyboard.m"));
        let session = controls.editor.as_ref().unwrap();
        let added = session
            .data
            .buttons()
            .iter()
            .find(|button| button.action == Some(Action::Key("key.keyboard.m".into())));
        assert!(added.is_some(), "the captured key must be a real binding");
        assert!(!session.ui.capturing);
        let _ = std::fs::remove_file(&controls.path);
    }

    #[test]
    fn layout_files_round_trip_through_export_and_import() {
        let mut controls = temp_controls("transfer");
        controls.viewport(1920, 1080);
        controls.open_editor();
        let mut session = controls.editor.take().unwrap();
        session.data.create("Exported").unwrap();
        assert!(controls.apply_editor(&mut session, Act::Save));
        let bytes = controls.export_layout().unwrap();
        let mut other = temp_controls("transfer-import");
        assert!(other.import_layout(&bytes).is_ok());
        assert_eq!(other.layouts.items.len(), 2);
        assert!(other.import_layout(b"{}").is_err());
        let _ = std::fs::remove_file(&controls.path);
        let _ = std::fs::remove_file(&other.path);
    }

    #[test]
    fn the_editor_fullscreen_panel_keeps_every_button_inside_the_surface() {
        let mut controls = temp_controls("geometry");
        controls.viewport(2400, 1080);
        controls.open_editor();
        let surface = controls.surface();
        let session = controls.editor.as_ref().unwrap();
        for widget in session.ui.widgets(&session.data, surface[0], surface[1]) {
            assert!(widget.rect[0] >= 0.0 && widget.rect[1] >= 0.0);
            assert!(widget.rect[0] + widget.rect[2] <= surface[0] + 0.5);
            assert!(widget.rect[1] + widget.rect[3] <= surface[1] + 0.5);
        }
        let _ = std::fs::remove_file(&controls.path);
    }

    #[test]
    fn hotbar_slots_and_the_more_button_keep_their_geometry() {
        assert_eq!(hotbar_slot(400.0, 225.0, [200.0, 220.0]), Some(4));
        assert_eq!(hotbar_slot(400.0, 225.0, [200.0, 100.0]), None);
        let rect = more_rect(400.0, 225.0, false);
        let inside = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
        assert!(more_at(400.0, 225.0, inside, false));
        assert!(!more_at(400.0, 225.0, [5.0, 5.0], false));
    }

    #[test]
    fn camera_sensitivity_output_stays_finite_for_odd_surfaces() {
        let settings = Settings::default();
        for (width, height) in [(1, 1), (1920, 1080), (1080, 2400)] {
            let delta = camera_delta([17.0, -9.0], width, height, &settings);
            assert!(delta.iter().all(|value| value.is_finite()));
        }
    }
}
