//! Versioned, independent CRAFTIX control layouts.
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Behavior { #[default] Hold, Toggle, Single }
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Id,
    pub action: Option<Action>,
    pub circle: Circle,
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
}
impl Button {
    pub fn valid(&self) -> bool {
        self.circle.valid() && self.opacity.is_finite() && (0.0..=1.0).contains(&self.opacity)
            && self.label.chars().count() <= 32
            && self.icon_rgba_16.as_deref().is_none_or(super::png_icon::valid)
            && self.action.as_ref().is_none_or(|a| match a {
                Action::Key(key) => crate::keybinds::key_code(key).is_some(),
                Action::Chord { modifiers, key } => valid_chord(modifiers, key),
                Action::Binding(name) => dispatchable_binding(name),
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
        let mut out=self.clone();
        if !width.is_finite() || !height.is_finite() || width<=0.0 || height<=0.0 { return out; }
        let aspect=width/height;
        let hotbar=[width/2.0-90.0,height-22.0,width/2.0+90.0,height];
        for button in &mut out.buttons {
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
        out
    }
    pub fn hit(&self, point: [f32; 2], aspect: f32) -> Option<&Button> {
        self.hit_for(point, aspect, false)
    }
    pub fn hit_for(&self, point: [f32; 2], aspect: f32, inventory: bool) -> Option<&Button> {
        self.buttons.iter().rev().find(|b| b.enabled && b.visibility.permits(inventory) && b.opacity > 0.0 && b.circle.contains(point, aspect))
    }
}
impl Default for Layout {
    fn default() -> Self {
        let defs = [
            (Id::Joystick,None,0.11,0.70,0.15,Behavior::Hold),
            (Id::Sprint,Some(Action::Sprint),0.275,0.73,0.05,Behavior::Hold),
            (Id::Attack,Some(Action::Attack),0.82,0.50,0.075,Behavior::Hold),
            (Id::Use,Some(Action::Use),0.82,0.70,0.085,Behavior::Hold),
            (Id::Jump,Some(Action::Jump),0.94,0.50,0.065,Behavior::Hold),
            (Id::Sneak,Some(Action::Sneak),0.94,0.70,0.065,Behavior::Hold),
            (Id::Inventory,Some(Action::Inventory),0.74,0.94,0.04,Behavior::Single),
            (Id::More,Some(Action::More),0.26,0.94,0.04,Behavior::Single),
            (Id::Chat,Some(Action::Chat),0.40,0.05,0.035,Behavior::Single),
            (Id::Pause,Some(Action::Pause),0.46,0.05,0.035,Behavior::Single),
            (Id::Settings,Some(Action::Settings),0.52,0.05,0.035,Behavior::Single),
            (Id::Players,Some(Action::Players),0.58,0.05,0.035,Behavior::Hold),
            (Id::Perspective,Some(Action::Perspective),0.64,0.05,0.035,Behavior::Single),
        ];
        Self { name: "Standard".into(), joystick_dead_zone: 0.12, joystick_sensitivity: 1.0,
            camera_sensitivity: 1.0, buttons: defs.into_iter().map(|(id, action, x,y,r,behavior)| Button {
                id, action, circle: Circle { center: [x,y], radius: r }, opacity: 0.85,
                behavior, icon: Icon::Text, icon_rgba_16: None, enabled: true, visibility: Visibility::Always, label: String::new(),
            }).collect() }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcut_parser_is_bounded_validated_and_round_trips() {
        for text in ["Ctrl + M", "Shift+A", "Alt+K", "key.keyboard.right.control+key.keyboard.m"] {
            let action=parse_key_action(text).unwrap();
            let json=serde_json::to_string(&action).unwrap();
            let restored: Action=serde_json::from_str(&json).unwrap();
            assert_eq!(action,restored);
            if let Action::Chord { modifiers,key }=restored { assert!(valid_chord(&modifiers,&key)); }
            else { panic!("expected chord"); }
        }
        for text in ["Ctrl+", "+M", "Ctrl+Ctrl+M", "Alt+NotAKey", "Ctrl+Shift+Alt+Ctrl+M", "Ctrl+Shift"] {
            assert!(parse_key_action(text).is_none(),"{text}");
        }
        assert_eq!(parse_key_action("A"),Some(Action::Key("key.keyboard.a".into())));
    }
    #[test]
    fn default_buttons_have_valid_geometry_and_circular_hit_boxes() {
        let layout = Layout::default();
        assert!(layout.valid());
        let jump = layout.buttons.iter().find(|b| b.id == Id::Jump).unwrap();
        let aspect = 16.0/9.0;
        assert_eq!(layout.hit(jump.circle.center, aspect).unwrap().id, Id::Jump);
        let corner = [jump.circle.center[0] + jump.circle.radius * 0.9 / aspect, jump.circle.center[1] + jump.circle.radius * 0.9];
        assert_ne!(layout.hit(corner, aspect).map(|b| b.id), Some(Id::Jump));
    }
    #[test]
    fn resolved_layout_keeps_controls_off_a_real_small_screen_hotbar() {
        let saved=Layout::default();
        for (width,height) in [(320.0_f32,240.0_f32),(427.0,240.0),(600.0,270.0)] {
            let resolved=saved.resolved(width,height);
            for b in &resolved.buttons {
                let x=b.circle.center[0]*width; let y=b.circle.center[1]*height; let r=b.circle.radius*height;
                let closest_x=x.clamp(width/2.0-90.0,width/2.0+90.0);
                let closest_y=y.clamp(height-22.0,height);
                assert!((x-closest_x).hypot(y-closest_y)>=r-0.001,"{:?} at {width}x{height}",b.id);
                assert!(x-r>=-0.001 && x+r<=width+0.001 && y-r>=-0.001 && y+r<=height+0.001);
            }
        }
        assert_eq!(saved,Layout::default());
    }
    #[test]
    fn factory_movement_buttons_do_not_overlap_on_phones() {
        let layout = Layout::default();
        for aspect in [4.0/3.0, 16.0/9.0, 20.0/9.0] {
            for pair in [(Id::Joystick, Id::Sprint), (Id::Attack, Id::Use), (Id::Attack, Id::Jump), (Id::Use, Id::Sneak), (Id::Jump, Id::Sneak)] {
                let a = layout.buttons.iter().find(|b| b.id == pair.0).unwrap().circle;
                let b = layout.buttons.iter().find(|b| b.id == pair.1).unwrap().circle;
                let d = ((a.center[0]-b.center[0])*aspect).hypot(a.center[1]-b.center[1]);
                assert!(d >= a.radius+b.radius, "{pair:?} at aspect {aspect}");
            }
        }
    }
}
