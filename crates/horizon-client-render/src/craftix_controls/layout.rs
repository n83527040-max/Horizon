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
    /// An existing game mapping, such as key.attack or key.hotbar.1.
    Binding(String),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Behavior { #[default] Hold, Toggle, Single }
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
            && self.action.as_ref().is_none_or(|a| match a {
                Action::Key(key) => crate::keybinds::key_code(key).is_some(),
                Action::Binding(name) => crate::keybinds::MAPPINGS.iter().any(|mapping| mapping.0 == name),
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
                behavior, icon: Icon::Text, enabled: true, visibility: Visibility::Always, label: String::new(),
            }).collect() }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
