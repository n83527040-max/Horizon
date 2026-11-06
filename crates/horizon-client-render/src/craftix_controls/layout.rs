//! Versioned, independent CRAFTIX control layouts.
//!
//! The geometry and interaction model follows the widget contract of
//! Zalith Launcher 2's `LayerController` module (GPL-3.0): anchored
//! positions, sizes expressed as a fraction of screen height, press /
//! swipable / toggle behaviors with out-of-bounds release and swipe
//! chains, penetrable widgets that let the camera keep the pointer, and
//! collapsible widget groups. The event vocabulary stays CRAFTIX's own
//! engine actions; the joystick stays analog because the engine consumes
//! stick vectors natively (the reference drives Java key events instead).
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Id {
    Joystick, Sprint, Jump, Sneak, Attack, Use, Inventory, More,
    Chat, Pause, Settings, Players, Perspective, Custom(u32),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Sprint, Jump, Sneak, Attack, Use, Inventory, More, Chat, Pause, Settings, Players, Perspective,
    /// The engine translates this physical-key token; no extra function is implied.
    Key(String),
    /// Ordered physical modifiers followed by a non-modifier key.
    Chord { modifiers: Vec<String>, key: String },
    /// An existing game mapping, such as key.attack or key.hotbar.1.
    Binding(String),
    /// Toggle a widget group's visibility (LayerController switch_layer).
    Group(u16),
    /// Open the system keyboard for the focused editor.
    Keyboard,
}

/// Parse a key or an ordered physical shortcut typed in the editor.
/// This models keyboard edges, not a promise that a particular mod supports
/// modifier-based bindings.
pub fn parse_key_action(text: &str) -> Option<Action> {
    fn canonical(part: &str) -> String {
        let part = part.trim().to_ascii_lowercase();
        if part.starts_with("key.") { part } else { format!("key.keyboard.{part}") }
    }
    fn modifier(part: &str) -> Option<&'static str> {
        match part.trim().to_ascii_lowercase().as_str() {
            "ctrl" | "control" | "key.keyboard.left.control" => Some("key.keyboard.left.control"),
            "key.keyboard.right.control" => Some("key.keyboard.right.control"),
            "shift" | "key.keyboard.left.shift" => Some("key.keyboard.left.shift"),
            "key.keyboard.right.shift" => Some("key.keyboard.right.shift"),
            "alt" | "key.keyboard.left.alt" => Some("key.keyboard.left.alt"),
            "key.keyboard.right.alt" => Some("key.keyboard.right.alt"),
            _ => None,
        }
    }
    let parts: Vec<_> = text.split('+').collect();
    if parts.len() == 1 {
        let key = canonical(parts[0]);
        return crate::keybinds::key_code(&key).map(|_| Action::Key(key));
    }
    if parts.len() > 4 { return None; }
    let mut modifiers = Vec::new();
    for part in &parts[..parts.len()-1] {
        let name = modifier(part)?.to_owned();
        if modifiers.contains(&name) { return None; }
        modifiers.push(name);
    }
    let last = parts[parts.len()-1].trim();
    if modifier(last).is_some() { return None; }
    let key = canonical(last);
    crate::keybinds::key_code(&key)?;
    Some(Action::Chord { modifiers, key })
}

pub fn valid_chord(modifiers: &[String], key: &str) -> bool {
    if modifiers.is_empty() || modifiers.len() > 3 { return false; }
    let text = format!("{}+{key}", modifiers.join("+"));
    matches!(parse_key_action(&text), Some(Action::Chord { modifiers: parsed, key: parsed_key }) if parsed == modifiers && parsed_key == key)
}

/// Interaction behavior model after Zalith's `InteractionBehavior`:
/// `Hold` (press) keeps pressing outside the widget, `Swipe` releases
/// when the pointer leaves the widget and accepts swipe chains, `Toggle`
/// flips on press and blocks swipe chains. `Single` is a CRAFTIX edge
/// pulse kept for compatibility with existing layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Behavior { #[default] Hold, Toggle, Single, Swipe }
impl Behavior {
    /// Finger leaving the widget bounds releases it (Swipable).
    pub fn release_on_out_of_bounds(self) -> bool { matches!(self, Self::Swipe) }
    /// Another pointer may slide into this widget while a chain is active.
    pub fn can_be_swiped_to(self) -> bool { matches!(self, Self::Swipe) }
    /// An active widget of this kind stops swipe-chain propagation (Toggle).
    pub fn blocks_swipe_chain(self) -> bool { matches!(self, Self::Toggle) }
}
/// The existing Android action dispatcher uses the mapping's default engine
/// input; entries whose default is Unknown would otherwise be inert.
pub fn dispatchable_binding(name: &str) -> bool {
    crate::keybinds::MAPPINGS.iter().any(|m| m.0 == name && m.1 != "key.keyboard.unknown")
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Icon { #[default] Text, Map, Sword, Use, Jump, Sneak, Sprint }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility { #[default] Always, Gameplay, Inventory }
impl Visibility {
    pub fn permits(self, inventory: bool) -> bool {
        match self { Self::Always => true, Self::Gameplay => !inventory, Self::Inventory => inventory }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// Center in surface fractions; radius is a fraction of surface height.
    pub center: [f32; 2],
    pub radius: f32,
}
impl Circle {
    pub fn contains(self, point: [f32; 2], aspect: f32) -> bool {
        let dx = (point[0] - self.center[0]) * aspect;
        let dy = point[1] - self.center[1];
        dx * dx + dy * dy <= self.radius * self.radius
    }
    pub fn valid(self) -> bool {
        self.center.into_iter().all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
            && self.radius.is_finite() && (0.02..=0.24).contains(&self.radius)
    }
}
/// Zalith-style rectangle: the anchor is a fraction of the *available*
/// space (0 = flush left/top, 1 = flush right/bottom) and both sizes are a
/// fraction of screen height, so resizing never pushes a widget off-screen
/// and neighbouring anchors stay clear of each other.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub anchor: [f32; 2],
    pub size: [f32; 2],
}
impl Rect {
    pub fn valid(self) -> bool {
        self.anchor.into_iter().all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
            && self.size.into_iter().all(|v| v.is_finite() && (0.03..=0.45).contains(&v))
    }
    /// Pixel-space box on a `width x height` surface.
    pub fn box_px(self, width: f32, height: f32) -> [f32; 4] {
        let w = (self.size[0] * height).min(width);
        let h = (self.size[1] * height).min(height);
        let x = (width - w) * self.anchor[0];
        let y = (height - h) * self.anchor[1];
        [x, y, w, h]
    }
    pub fn contains(self, point: [f32; 2], aspect: f32) -> bool {
        if !aspect.is_finite() || aspect <= 0.0 { return false; }
        // Surface fractions: width fraction of a height-sized widget.
        let w = (self.size[0] / aspect).min(1.0);
        let h = self.size[1].min(1.0);
        let x = (1.0 - w) * self.anchor[0];
        let y = (1.0 - h) * self.anchor[1];
        point[0] >= x && point[0] <= x + w && point[1] >= y && point[1] <= y + h
    }
    pub fn center(self, aspect: f32) -> [f32; 2] {
        let aspect = if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 };
        let w = (self.size[0] / aspect).min(1.0);
        let h = self.size[1].min(1.0);
        [(1.0 - w) * self.anchor[0] + w * 0.5, (1.0 - h) * self.anchor[1] + h * 0.5]
    }
}
/// Geometry accessor: new widgets carry a `rect`; layouts written before
/// the LayerController port keep their circle. Both are hit-tested and
/// painted from the same resolved copy of the layout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Id,
    pub action: Option<Action>,
    pub circle: Circle,
    /// Preferred rectangle geometry; absent in schema-1 circle layouts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<Rect>,
    pub opacity: f32,
    pub behavior: Behavior,
    pub icon: Icon,
    /// Embedded 16x16 RGBA PNG preview; absent in older layouts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_rgba_16: Option<String>,
    pub enabled: bool,
    /// Missing from schema-1 files made before contextual visibility was added.
    #[serde(default)]
    pub visibility: Visibility,
    pub label: String,
    /// Touch passes to the camera while the widget is pressed (isPenetrable).
    #[serde(default)]
    pub penetrable: bool,
    /// Widget group; group 0 is always shown, others toggle via Action::Group.
    #[serde(default)]
    pub group: u16,
}
impl Button {
    pub fn contains(&self, point: [f32; 2], aspect: f32) -> bool {
        match self.rect {
            Some(rect) => rect.contains(point, aspect),
            None => self.circle.contains(point, aspect),
        }
    }
    /// Center in surface fractions for both geometries.
    pub fn center(&self, aspect: f32) -> [f32; 2] {
        self.rect.map_or(self.circle.center, |r| r.center(aspect))
    }
    /// Radius-like half extent in height fractions (joystick math, margins).
    pub fn half_height(&self) -> f32 {
        self.rect.map_or(self.circle.radius, |r| r.size[1] * 0.5)
    }
    /// Move keeping the current geometry: anchors recompute from the
    /// requested surface-fraction center.
    pub fn set_center(&mut self, point: [f32; 2], aspect: f32) {
        if !aspect.is_finite() || aspect <= 0.0 { return; }
        if let Some(rect) = &mut self.rect {
            let w = (rect.size[0] / aspect).min(1.0);
            let h = rect.size[1].min(1.0);
            rect.anchor[0] = ((point[0] - w * 0.5) / (1.0 - w).max(1e-6)).clamp(0.0, 1.0);
            rect.anchor[1] = ((point[1] - h * 0.5) / (1.0 - h).max(1e-6)).clamp(0.0, 1.0);
        } else {
            let rx = (self.circle.radius / aspect).min(0.5);
            let ry = self.circle.radius.min(0.5);
            self.circle.center = [point[0].clamp(rx, 1.0 - rx), point[1].clamp(ry, 1.0 - ry)];
        }
    }
    /// Scale the widget about its center (editor size slider).
    pub fn scale(&mut self, factor: f32) {
        if let Some(rect) = &mut self.rect {
            rect.size = [
                (rect.size[0] * factor).clamp(0.03, 0.45),
                (rect.size[1] * factor).clamp(0.03, 0.45),
            ];
        } else {
            self.circle.radius = (self.circle.radius * factor).clamp(0.02, 0.24);
        }
    }
    /// Size readout for the editor (height fraction).
    pub fn size_value(&self) -> f32 { self.half_height() * 2.0 }
    /// Editor size slider: overall height fraction, scaling both axes.
    pub fn set_size_value(&mut self, value: f32) {
        let value = value.clamp(0.03, 0.45);
        let current = if self.rect.is_some() { self.size_value() } else { 0.0 };
        if self.rect.is_some() {
            let factor = value / current.max(1e-6);
            if let Some(rect) = &mut self.rect {
                rect.size = [
                    (rect.size[0] * factor).clamp(0.03, 0.45),
                    (rect.size[1] * factor).clamp(0.03, 0.45),
                ];
            }
        } else {
            self.circle.radius = (value * 0.5).clamp(0.02, 0.24);
        }
    }
    pub fn valid(&self) -> bool {
        let geometry_ok = match self.rect {
            Some(rect) => rect.valid(),
            None => self.circle.valid(),
        };
        geometry_ok && self.opacity.is_finite() && (0.0..=1.0).contains(&self.opacity)
            && self.label.chars().count() <= 32
            && self.icon_rgba_16.as_deref().is_none_or(super::png_icon::valid)
            && self.action.as_ref().is_none_or(|a| match a {
                Action::Key(key) => crate::keybinds::key_code(key).is_some(),
                Action::Chord { modifiers, key } => valid_chord(modifiers, key),
                Action::Binding(name) => dispatchable_binding(name),
                Action::Group(group) => *group <= 64,
                Action::Keyboard => true,
                _ => true,
            })
            && (self.id != Id::Joystick || self.action.is_none())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub buttons: Vec<Button>,
    pub joystick_dead_zone: f32,
    pub joystick_sensitivity: f32,
    pub camera_sensitivity: f32,
    /// Hidden widget groups; runtime-only (LayerController layer.hide).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub hidden_groups: BTreeSet<u16>,
}
impl Layout {
    pub fn valid(&self) -> bool {
        let name_len = self.name.trim().chars().count();
        let mut ids = BTreeSet::new();
        name_len > 0 && name_len <= 32 && self.buttons.len() <= 64
            && self.buttons.iter().all(|b| b.valid() && ids.insert(b.id))
            && self.joystick_dead_zone.is_finite() && (0.0..=0.5).contains(&self.joystick_dead_zone)
            && self.joystick_sensitivity.is_finite() && (0.25..=3.0).contains(&self.joystick_sensitivity)
            && self.camera_sensitivity.is_finite() && (0.25..=3.0).contains(&self.camera_sensitivity)
    }
    /// Resolve against the actual GUI-pixel hotbar without changing the saved
    /// layout. Painting and hit testing must use the same resolved copy.
    pub fn resolved(&self, width: f32, height: f32) -> Self {
        let mut out = Self { name: String::new(), buttons: Vec::new(), joystick_dead_zone: 0.0,
            joystick_sensitivity: 1.0, camera_sensitivity: 1.0, hidden_groups: BTreeSet::new() };
        self.resolve_into(&mut out, width, height);
        out
    }
    /// Reuse a host-owned buffer so per-frame resolution does not allocate a
    /// fresh button vector and labels every frame (frame-pacing aid).
    pub fn resolve_into(&self, out: &mut Self, width: f32, height: f32) {
        out.clone_from(self);
        self.apply_resolution(out, width, height);
    }
    fn apply_resolution(&self, out: &mut Self, width: f32, height: f32) {
        if !width.is_finite() || !height.is_finite() || width<=0.0 || height<=0.0 { return; }
        let aspect=width/height;
        let hotbar=[width/2.0-90.0,height-22.0,width/2.0+90.0,height];
        for button in &mut out.buttons {
            if let Some(rect) = &mut button.rect {
                let w = (rect.size[0] * height).min(width);
                let h = (rect.size[1] * height).min(height);
                let x = (width - w) * rect.anchor[0];
                let y = (height - h) * rect.anchor[1];
                if rect_overlap_hotbar(x, y, w, h, hotbar) {
                    rect.anchor[1] = (((hotbar[1] - h - 3.0) / (height - h).max(1.0))).clamp(0.0, 1.0);
                }
                continue;
            }
            let r=button.circle.radius.min(0.5).min(aspect*0.5);
            button.circle.radius=r;
            let rx=r/aspect;
            button.circle.center[0]=button.circle.center[0].clamp(rx,1.0-rx);
            button.circle.center[1]=button.circle.center[1].clamp(r,1.0-r);
            let x=button.circle.center[0]*width; let y=button.circle.center[1]*height; let radius=r*height;
            let closest_x=x.clamp(hotbar[0],hotbar[2]);
            let closest_y=y.clamp(hotbar[1],hotbar[3]);
            if (x-closest_x).hypot(y-closest_y)<radius {
                button.circle.center[1]=((hotbar[1]-radius-3.0)/height).clamp(r,1.0-r);
            }
        }
    }
    pub fn hit(&self, point: [f32; 2], aspect: f32) -> Option<&Button> {
        self.hit_for(point, aspect, false)
    }
    /// Only custom controls that are visible in inventories and do not touch
    /// the protected container/recipe area. The same layout must be used for
    /// painting and hit-testing, so a hidden button never steals inventory taps.
    pub fn inventory_overlay(&self, width: f32, height: f32, blocked: [f32; 4]) -> Layout {
        let mut layout=self.resolved(width,height);
        if !width.is_finite() || !height.is_finite() || width<=0.0 || height<=0.0 || !blocked.into_iter().all(f32::is_finite) {
            layout.buttons.clear(); return layout;
        }
        layout.buttons.retain(|button| {
            if !matches!(button.id, Id::Custom(_)) || !button.visibility.permits(true) { return false; }
            let (x, y, w, h) = match button.rect {
                Some(rect) => { let b = rect.box_px(width, height); (b[0], b[1], b[2], b[3]) }
                None => {
                    let x=button.circle.center[0]*width; let y=button.circle.center[1]*height;
                    let r=button.circle.radius*height;
                    (x - r, y - r, r * 2.0, r * 2.0)
                }
            };
            x+w<=blocked[0] || x>=blocked[2] || y+h<=blocked[1] || y>=blocked[3]
        });
        layout
    }
    pub fn hit_for(&self, point: [f32; 2], aspect: f32, inventory: bool) -> Option<&Button> {
        self.buttons.iter().rev().find(|b| b.enabled
            && !self.hidden_groups.contains(&b.group)
            && b.visibility.permits(inventory) && b.opacity > 0.0 && b.contains(point, aspect))
    }
}
fn rect_overlap_hotbar(x: f32, y: f32, w: f32, h: f32, hotbar: [f32; 4]) -> bool {
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let closest_x = cx.clamp(hotbar[0], hotbar[2]);
    let closest_y = cy.clamp(hotbar[1], hotbar[3]);
    (cx - closest_x).hypot(cy - closest_y) < (w.min(h) * 0.5)
}
impl Default for Layout {
    fn default() -> Self {
        // Port of Zalith Launcher 2's default_layout.json positions and
        // sizes (rects are fractions of screen height, anchors are
        // fractions of the available space) mapped onto CRAFTIX engine
        // actions. Group 1 is the collapsible top bar (GUI button).
        let defs: [(Id, Option<Action>, [f32; 2], [f32; 2], Behavior, u16, bool, &str); 16] = [
            (Id::Pause,    Some(Action::Pause),      [0.0, 0.0],       [0.2289, 0.11],  Behavior::Hold,   1, false, "Esc"),
            (Id::Custom(1),Some(Action::Key("key.keyboard.f3".into())), [0.1154, 0.0], [0.2289, 0.11], Behavior::Single, 1, false, "F3"),
            (Id::Perspective, Some(Action::Perspective), [0.2308, 0.0],  [0.2289, 0.11],  Behavior::Single, 1, false, "F5"),
            (Id::Players,  Some(Action::Players),     [0.0, 0.1239],    [0.2289, 0.11],  Behavior::Hold,   1, false, "Tab"),
            (Id::Chat,     Some(Action::Chat),        [0.1154, 0.1239],[0.2289, 0.11],  Behavior::Single, 1, false, "T"),
            (Id::Custom(2),Some(Action::Keyboard),    [0.2308, 0.1239],[0.2289, 0.11],  Behavior::Single, 1, false, "IME"),
            (Id::More,     Some(Action::More),        [1.0, 0.1239],   [0.2289, 0.11],  Behavior::Single, 0, false, "More"),
            (Id::Custom(3),Some(Action::Group(1)),    [1.0, 0.0],       [0.2289, 0.11],  Behavior::Toggle, 0, false, "GUI"),
            (Id::Custom(4),Some(Action::Key("key.keyboard.left.control".into())), [0.0846, 0.3105], [0.2621, 0.11], Behavior::Hold, 0, false, "Ctrl"),
            (Id::Custom(5),Some(Action::Key("key.keyboard.left.alt".into())),    [0.219, 0.3105],  [0.2621, 0.11], Behavior::Hold, 0, false, "Alt"),
            (Id::Sprint,   Some(Action::Sprint),      [0.0846, 0.44],   [0.16, 0.11],    Behavior::Toggle, 0, false, "Sprint"),
            (Id::Attack,   Some(Action::Attack),      [0.7614, 0.548],  [0.215, 0.138],  Behavior::Hold,   0, true,  "PRI"),
            (Id::Use,      Some(Action::Use),         [0.869, 0.548],   [0.215, 0.138],  Behavior::Hold,   0, true,  "SEC"),
            (Id::Sneak,    Some(Action::Sneak),       [0.7614, 0.708],  [0.215, 0.138],  Behavior::Toggle, 0, false, "Sneak"),
            (Id::Jump,     Some(Action::Jump),        [0.869, 0.708],   [0.215, 0.138],  Behavior::Hold,   0, false, "Jump"),
            (Id::Inventory,Some(Action::Inventory),   [0.733, 1.0],     [0.12, 0.12],    Behavior::Single, 0, false, "E"),
        ];
        let mut buttons: Vec<Button> = defs.into_iter().map(|(id, action, anchor, size, behavior, group, penetrable, label)| Button {
            id, action,
            rect: Some(Rect { anchor, size }),
            // Placeholder only: rect geometry wins for these widgets.
            circle: Circle { center: Rect { anchor, size }.center(16.0 / 9.0), radius: size[1] * 0.5 },
            opacity: 0.85, behavior, icon: Icon::Text, icon_rgba_16: None,
            enabled: true, visibility: Visibility::Gameplay, label: label.into(),
            penetrable, group,
        }).collect();
        buttons.insert(0, Button {
            id: Id::Joystick, action: None,
            rect: Some(Rect { anchor: [0.1121, 0.7953], size: [0.3417, 0.3417] }),
            circle: Circle { center: Rect { anchor: [0.1121, 0.7953], size: [0.3417, 0.3417] }.center(16.0 / 9.0), radius: 0.17 },
            opacity: 0.85, behavior: Behavior::Hold, icon: Icon::Text, icon_rgba_16: None,
            enabled: true, visibility: Visibility::Gameplay, label: "Move".into(),
            penetrable: false, group: 0,
        });
        // Analog engine:0.25 keeps Zalith's generous centre guard while
        // staying responsive for stick vectors (their 0.5 guards 8-way
        // key emulation; documented deviation in the port notes).
        Self { name: "Zalith-style".into(), joystick_dead_zone: 0.25, joystick_sensitivity: 1.0,
            camera_sensitivity: 1.0, hidden_groups: BTreeSet::new(), buttons }
    }
}
