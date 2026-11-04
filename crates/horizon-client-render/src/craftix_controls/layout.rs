//! Versioned, independent CRAFTIX control layouts.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Every on-screen control the replacement HUD knows about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Id {
    Joystick,
    Sprint,
    Jump,
    Sneak,
    Attack,
    Use,
    Inventory,
    More,
    Chat,
    Pause,
    Settings,
    Players,
    Perspective,
    Custom(u32),
}
impl Id {
    /// The fixed controls the factory layout ships with.
    pub const BUILTIN: [Id; 13] = [
        Id::Joystick,
        Id::Sprint,
        Id::Jump,
        Id::Sneak,
        Id::Attack,
        Id::Use,
        Id::Inventory,
        Id::More,
        Id::Chat,
        Id::Pause,
        Id::Settings,
        Id::Players,
        Id::Perspective,
    ];
    pub fn is_custom(self) -> bool {
        matches!(self, Id::Custom(_))
    }
    /// Name shown in the editor; custom buttons use their own label.
    pub fn title(self) -> &'static str {
        match self {
            Id::Joystick => "Joystick",
            Id::Sprint => "Sprint",
            Id::Jump => "Jump",
            Id::Sneak => "Sneak",
            Id::Attack => "Attack",
            Id::Use => "Use",
            Id::Inventory => "Inventory",
            Id::More => "More",
            Id::Chat => "Chat",
            Id::Pause => "Pause",
            Id::Settings => "Settings",
            Id::Players => "Players",
            Id::Perspective => "Perspective",
            Id::Custom(_) => "Custom",
        }
    }
    pub fn from_title(title: &str) -> Option<Id> {
        Id::BUILTIN
            .into_iter()
            .find(|id| id.title().eq_ignore_ascii_case(title.trim()))
    }
}

/// A binding the engine can really act on. Nothing else is accepted.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Sprint,
    Jump,
    Sneak,
    Attack,
    Use,
    Inventory,
    More,
    Chat,
    Pause,
    Settings,
    Players,
    Perspective,
    /// A physical key (`key.keyboard.m`) or an engine mapping (`action:key.jump`).
    Key(String),
}
impl Action {
    /// The engine token for bindings that are delivered as key edges.
    pub fn token(&self) -> Option<&str> {
        match self {
            Action::Key(key) => Some(key),
            Action::Players => Some("action:key.playerlist"),
            _ => None,
        }
    }
    pub fn label(&self) -> String {
        match self {
            Action::Key(key) => {
                if let Some(mapping) = key.strip_prefix("action:") {
                    crate::text::Text::translate(mapping, &[]).plain()
                } else {
                    crate::keybinds::display_name(key).plain()
                }
            }
            Action::Sprint => "Sprint".into(),
            Action::Jump => "Jump".into(),
            Action::Sneak => "Sneak".into(),
            Action::Attack => "Attack".into(),
            Action::Use => "Use".into(),
            Action::Inventory => "Inventory".into(),
            Action::More => "More".into(),
            Action::Chat => "Chat".into(),
            Action::Pause => "Pause".into(),
            Action::Settings => "Settings".into(),
            Action::Players => "Players".into(),
            Action::Perspective => "Perspective".into(),
        }
    }
}

/// Whether a token names a real key or a real engine mapping.
pub fn valid_token(token: &str) -> bool {
    crate::keybinds::key_code(token).is_some()
        || token
            .strip_prefix("action:")
            .is_some_and(|name| crate::keybinds::MAPPINGS.iter().any(|m| m.0 == name))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Behavior {
    #[default]
    Hold,
    Toggle,
    Single,
}

/// Presentation only: the bound action stays authoritative.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Icon {
    #[default]
    Text,
    Map,
    Sword,
    Use,
    Jump,
    Sneak,
    Sprint,
    Inventory,
    Chat,
    Pause,
    Settings,
    Players,
    Perspective,
    More,
}
impl Icon {
    pub const ALL: [Icon; 14] = [
        Icon::Text,
        Icon::Map,
        Icon::Sword,
        Icon::Use,
        Icon::Jump,
        Icon::Sneak,
        Icon::Sprint,
        Icon::Inventory,
        Icon::Chat,
        Icon::Pause,
        Icon::Settings,
        Icon::Players,
        Icon::Perspective,
        Icon::More,
    ];
    pub fn next(self) -> Icon {
        let index = Icon::ALL.iter().position(|i| *i == self).unwrap_or(0);
        Icon::ALL[(index + 1) % Icon::ALL.len()]
    }
    pub fn label(self) -> &'static str {
        match self {
            Icon::Text => "Text",
            Icon::Map => "Map",
            Icon::Sword => "Sword",
            Icon::Use => "Use",
            Icon::Jump => "Jump",
            Icon::Sneak => "Sneak",
            Icon::Sprint => "Sprint",
            Icon::Inventory => "Inventory",
            Icon::Chat => "Chat",
            Icon::Pause => "Pause",
            Icon::Settings => "Settings",
            Icon::Players => "Players",
            Icon::Perspective => "Perspective",
            Icon::More => "More",
        }
    }
}

/// The screen a button belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Always,
    Gameplay,
    Inventory,
}
impl Visibility {
    pub fn next(self) -> Self {
        match self {
            Visibility::Always => Visibility::Gameplay,
            Visibility::Gameplay => Visibility::Inventory,
            Visibility::Inventory => Visibility::Always,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Visibility::Always => "Always",
            Visibility::Gameplay => "Gameplay",
            Visibility::Inventory => "Inventory",
        }
    }
    pub fn permits(self, inventory: bool) -> bool {
        match self {
            Visibility::Always => true,
            Visibility::Gameplay => !inventory,
            Visibility::Inventory => inventory,
        }
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
        self.center
            .into_iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
            && self.radius.is_finite()
            && (0.015..=0.24).contains(&self.radius)
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
    pub label: String,
    #[serde(default)]
    pub visibility: Visibility,
}
impl Button {
    pub fn new(id: Id, action: Option<Action>, center: [f32; 2], radius: f32) -> Self {
        Self {
            id,
            action,
            circle: Circle { center, radius },
            opacity: 0.85,
            behavior: Behavior::Hold,
            icon: Icon::Text,
            enabled: true,
            label: String::new(),
            visibility: Visibility::Always,
        }
    }
    pub fn valid(&self) -> bool {
        self.circle.valid()
            && self.opacity.is_finite()
            && (0.0..=1.0).contains(&self.opacity)
            && self.label.chars().count() <= 32
            && !self.label.chars().any(char::is_control)
            && self
                .action
                .as_ref()
                .is_none_or(|action| match action {
                    Action::Key(token) => valid_token(token),
                    _ => true,
                })
            && (self.id != Id::Joystick || self.action.is_none())
    }
    /// Custom buttons need a real binding; built-in ones always work.
    pub fn operable(&self) -> bool {
        self.enabled
            && self.opacity > 0.0
            && (self.action.is_some() || self.id == Id::Joystick)
            && (self.id != Id::Joystick || self.action.is_none())
    }
    pub fn label_text(&self) -> String {
        if !self.label.trim().is_empty() {
            self.label.clone()
        } else if self.id.is_custom() {
            self.action.as_ref().map(Action::label).unwrap_or_else(|| "Button".into())
        } else {
            self.id.title().to_string()
        }
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
        name_len > 0
            && name_len <= 32
            && self.buttons.len() <= 64
            && self.buttons.iter().all(|b| b.valid() && ids.insert(b.id))
            && self.joystick_dead_zone.is_finite()
            && (0.0..=0.5).contains(&self.joystick_dead_zone)
            && self.joystick_sensitivity.is_finite()
            && (0.25..=3.0).contains(&self.joystick_sensitivity)
            && self.camera_sensitivity.is_finite()
            && (0.25..=3.0).contains(&self.camera_sensitivity)
    }
    /// The topmost operable button under `point` for the given screen.
    pub fn hit_in(&self, point: [f32; 2], aspect: f32, inventory: bool) -> Option<&Button> {
        self.buttons.iter().rev().find(|b| {
            b.operable()
                && b.visibility.permits(inventory)
                && b.circle.contains(point, aspect)
        })
    }
    pub fn hit(&self, point: [f32; 2], aspect: f32) -> Option<&Button> {
        self.hit_in(point, aspect, false)
    }
    pub fn button(&self, id: Id) -> Option<&Button> {
        self.buttons.iter().find(|b| b.id == id)
    }
    pub fn button_mut(&mut self, id: Id) -> Option<&mut Button> {
        self.buttons.iter_mut().find(|b| b.id == id)
    }
    pub fn contains_action(&self, action: &Action) -> bool {
        self.buttons.iter().any(|b| b.action.as_ref() == Some(action))
    }
    /// Mirror horizontally for left-handed play, keeping every radius.
    pub fn mirror(&mut self) {
        for button in &mut self.buttons {
            button.circle.center[0] = (1.0 - button.circle.center[0]).clamp(0.0, 1.0);
        }
    }
    pub fn mirrored(mut self) -> Self {
        self.mirror();
        self
    }
    /// The factory layout: round glass controls, a joystick and a five-button
    /// top bar. Offsets are fractions of the surface so every aspect works.
    pub fn factory(name: &str) -> Self {
        let mut buttons = vec![
            Button::new(Id::Joystick, None, [0.13, 0.68], 0.16),
            Button::new(Id::Sprint, Some(Action::Sprint), [0.29, 0.74], 0.055),
            Button::new(Id::Jump, Some(Action::Jump), [0.93, 0.54], 0.062),
            Button::new(Id::Sneak, Some(Action::Sneak), [0.93, 0.72], 0.062),
            Button::new(Id::Attack, Some(Action::Attack), [0.80, 0.52], 0.075),
            Button::new(Id::Use, Some(Action::Use), [0.80, 0.70], 0.085),
            Button::new(Id::Inventory, Some(Action::Inventory), [0.72, 0.93], 0.042),
            Button::new(Id::More, Some(Action::More), [0.28, 0.93], 0.042),
            Button::new(Id::Chat, Some(Action::Chat), [0.40, 0.055], 0.032),
            Button::new(Id::Pause, Some(Action::Pause), [0.46, 0.055], 0.032),
            Button::new(Id::Settings, Some(Action::Settings), [0.52, 0.055], 0.032),
            Button::new(Id::Players, Some(Action::Players), [0.58, 0.055], 0.032),
            Button::new(Id::Perspective, Some(Action::Perspective), [0.64, 0.055], 0.032),
        ];
        for button in &mut buttons {
            if button.id == Id::More || button.id == Id::Inventory {
                button.behavior = Behavior::Single;
            }
            if matches!(button.id, Id::Chat | Id::Pause | Id::Settings | Id::Perspective) {
                button.behavior = Behavior::Single;
            }
        }
        Self {
            name: name.into(),
            buttons,
            joystick_dead_zone: 0.12,
            joystick_sensitivity: 1.0,
            camera_sensitivity: 1.0,
        }
    }
}
impl Default for Layout {
    fn default() -> Self {
        Self::factory("Standard")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_buttons_have_valid_geometry_and_circular_hit_boxes() {
        let layout = Layout::default();
        assert!(layout.valid());
        let jump = layout.button(Id::Jump).unwrap();
        let aspect = 16.0 / 9.0;
        assert_eq!(layout.hit(jump.circle.center, aspect).unwrap().id, Id::Jump);
        let corner = [
            jump.circle.center[0] + jump.circle.radius * 0.9 / aspect,
            jump.circle.center[1] + jump.circle.radius * 0.9,
        ];
        assert_ne!(layout.hit(corner, aspect).map(|b| b.id), Some(Id::Jump));
    }
    #[test]
    fn factory_buttons_never_overlap_on_phones() {
        let layout = Layout::default();
        // Landscape is the gameplay orientation; the host squeezes the set on
        // anything narrower, so only landscape aspects are asserted here.
        for aspect in [4.0 / 3.0, 16.0 / 9.0, 20.0 / 9.0] {
            for (i, a) in layout.buttons.iter().enumerate() {
                for b in &layout.buttons[i + 1..] {
                    let d = ((a.circle.center[0] - b.circle.center[0]) * aspect)
                        .hypot(a.circle.center[1] - b.circle.center[1]);
                    assert!(
                        d >= a.circle.radius + b.circle.radius,
                        "{:?} and {:?} overlap at aspect {aspect}",
                        a.id,
                        b.id
                    );
                }
            }
        }
    }
    #[test]
    fn every_factory_button_stays_inside_the_surface() {
        let layout = Layout::default();
        for aspect in [4.0 / 3.0, 16.0 / 9.0, 20.0 / 9.0] {
            for b in &layout.buttons {
                assert!(b.circle.center[0] - b.circle.radius / aspect >= 0.0, "{:?}", b.id);
                assert!(b.circle.center[0] + b.circle.radius / aspect <= 1.0, "{:?}", b.id);
                assert!(b.circle.center[1] - b.circle.radius >= 0.0, "{:?}", b.id);
                assert!(b.circle.center[1] + b.circle.radius <= 1.0, "{:?}", b.id);
            }
        }
    }
    #[test]
    fn mirroring_keeps_geometry_valid_and_left_handed() {
        let mut layout = Layout::default();
        let joystick = layout.button(Id::Joystick).unwrap().circle.center[0];
        layout.mirror();
        assert!(layout.valid());
        assert!((layout.button(Id::Joystick).unwrap().circle.center[0] - (1.0 - joystick)).abs() < 1e-6);
    }
    #[test]
    fn visibility_and_disabled_state_remove_hit_targets() {
        let mut layout = Layout::default();
        layout.button_mut(Id::Attack).unwrap().visibility = Visibility::Gameplay;
        let point = layout.button(Id::Attack).unwrap().circle.center;
        assert!(layout.hit_in(point, 16.0 / 9.0, true).is_none());
        assert_eq!(layout.hit_in(point, 16.0 / 9.0, false).unwrap().id, Id::Attack);
        layout.button_mut(Id::Attack).unwrap().visibility = Visibility::Always;
        assert_eq!(layout.hit_in(point, 16.0 / 9.0, true).unwrap().id, Id::Attack);
        layout.button_mut(Id::Attack).unwrap().enabled = false;
        assert!(layout.hit(point, 16.0 / 9.0).is_none());
    }
    #[test]
    fn invalid_bindings_and_joystick_actions_are_rejected() {
        let mut button = Button::new(Id::Custom(0), Some(Action::Key("key.keyboard.not-real".into())), [0.5, 0.5], 0.05);
        assert!(!button.valid());
        button.action = Some(Action::Key("key.keyboard.m".into()));
        assert!(button.valid());
        button.action = Some(Action::Key("action:key.playerlist".into()));
        assert!(button.valid());
        let mut joystick = Button::new(Id::Joystick, Some(Action::Jump), [0.2, 0.6], 0.16);
        assert!(!joystick.valid());
        joystick.action = None;
        assert!(joystick.valid());
    }
}
