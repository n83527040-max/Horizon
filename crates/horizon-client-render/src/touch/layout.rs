//! Per-profile touch layouts. Coordinates are normalized; sizes are fractions
//! of surface height, so a button stays square on wide phones and tablets.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Crosshair,
    Tap,
    Dpad,
}
impl Mode {
    pub const ALL: [Self; 3] = [Self::Crosshair, Self::Tap, Self::Dpad];
    pub fn label(self) -> &'static str {
        match self {
            Self::Crosshair => "Joystick & aim crosshair",
            Self::Tap => "Joystick & tap to interact",
            Self::Dpad => "D-Pad & tap to interact",
        }
    }
    pub const SCHEMES: [(Self, bool, &str); 4] = [
        (Self::Crosshair, false, "Joystick & aim crosshair"),
        (Self::Tap, false, "Joystick & tap to interact"),
        (Self::Dpad, false, "D-Pad & tap to interact"),
        (Self::Dpad, true, "D-Pad & aim crosshair"),
    ];
    pub fn index(self) -> usize {
        match self {
            Self::Crosshair => 0,
            Self::Tap => 1,
            Self::Dpad => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Move,
    Sprint,
    Jump,
    Sneak,
    Attack,
    Use,
    Chat,
    Pause,
    Settings,
    Inventory,
    Debug,
    Perspective,
    Offhand,
    Shield,
}
impl Control {
    pub const ALL: [Self; 14] = [
        Self::Move,
        Self::Sprint,
        Self::Jump,
        Self::Sneak,
        Self::Attack,
        Self::Use,
        Self::Chat,
        Self::Pause,
        Self::Settings,
        Self::Inventory,
        Self::Debug,
        Self::Perspective,
        Self::Offhand,
        Self::Shield,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Move => "Movement",
            Self::Sprint => "Sprint",
            Self::Jump => "Jump / fly up",
            Self::Sneak => "Sneak / fly down",
            Self::Attack => "Attack / mine",
            Self::Use => "Use / place",
            Self::Chat => "Chat",
            Self::Pause => "Pause",
            Self::Settings => "Touch settings",
            Self::Inventory => "Inventory",
            Self::Debug => "Debug overlay (F3)",
            Self::Perspective => "Camera perspective (F5)",
            Self::Offhand => "Swap hands",
            Self::Shield => "Raise shield",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Placement {
    pub center: [f32; 2],
    pub size: f32,
    pub opacity: f32,
}
impl Placement {
    pub(super) fn new(x: f32, y: f32, size: f32) -> Self {
        Self {
            center: [x, y],
            size,
            opacity: 0.65,
        }
    }
    pub fn rect(self, aspect: f32) -> [f32; 4] {
        let sx = self.size / aspect.max(0.1);
        [
            self.center[0] - sx / 2.0,
            self.center[1] - self.size / 2.0,
            sx,
            self.size,
        ]
    }
    pub fn sanitize(&mut self) {
        if !self.size.is_finite() {
            self.size = 0.15;
        }
        if !self.opacity.is_finite() {
            self.opacity = 0.65;
        }
        self.size = self.size.clamp(0.045, 0.45);
        self.opacity = self.opacity.clamp(0.0, 1.0);
        for c in &mut self.center {
            *c = if c.is_finite() {
                c.clamp(0.025, 0.975)
            } else {
                0.5
            };
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub split_controls: bool,
    pub contextual_interact: bool,
    pub left_handed: bool,
    pub sprint_with_joystick: bool,
    pub swap_jump_sneak: bool,
    pub sensitivity: f32,
    pub sensitivity_horizontal: f32,
    pub sensitivity_vertical: f32,
    pub offhand_button: bool,
    pub shield_button: bool,
    pub circle_size: f32,
    pub button_scale: f32,
    pub custom_buttons: Vec<super::CustomButton>,
    pub analog_movement: bool,
    pub sprint_button: bool,
    pub sprint_toggle: bool,
    pub inventory_controls: bool,
    pub virtual_mouse_sensitivity: f32,
    pub crosshair_gestures: bool,
    pub gyroscope: bool,
    pub gyro_sensitivity: f32,
    pub attack_indicator_left: bool,
    #[serde(deserialize_with = "deserialize_visibility")]
    pub button_visibility: [u8; 14],
    pub horizontal_margin: f32,
    pub vertical_margin: f32,
    #[serde(deserialize_with = "deserialize_layouts")]
    pub layouts: [[Placement; 14]; 3],
}
// Append new buttons without resetting a player's existing ten-button layout.
fn deserialize_layouts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<[[Placement; 14]; 3], D::Error> {
    let rows = Vec::<Vec<Placement>>::deserialize(deserializer)?;
    if rows.len() != 3 || rows.iter().any(|r| !matches!(r.len(), 10 | 12 | 14)) {
        return Err(serde::de::Error::custom(
            "expected three touch layouts of 10, 12 or 14 controls",
        ));
    }
    let mut layouts = Settings::default().layouts;
    for (output, input) in layouts.iter_mut().zip(rows) {
        output[..input.len()].copy_from_slice(&input);
    }
    Ok(layouts)
}
fn deserialize_visibility<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[u8; 14], D::Error> {
    let old = Vec::<u8>::deserialize(d)?;
    if !matches!(old.len(), 12 | 14) {
        return Err(serde::de::Error::custom(
            "expected 12 or 14 visibility values",
        ));
    }
    let mut values = [0; 14];
    values[..old.len()].copy_from_slice(&old);
    Ok(values)
}
/// Reference-inspired Android factory placement. Existing saved layouts are
/// never overwritten; the player can explicitly select the Standard preset.
#[cfg(any(target_os = "android", test))]
fn craftix_factory_layout(layouts: &mut [[Placement; 14]; 3]) {
    for mode in [Mode::Crosshair, Mode::Tap] {
        let row = &mut layouts[mode.index()];
        for (control, x, y, size) in [
            (Control::Move, 0.11, 0.70, 0.32),
            (Control::Sprint, 0.25, 0.73, 0.105),
            (Control::Attack, 0.82, 0.53, 0.18),
            (Control::Use, 0.80, 0.73, 0.18),
            (Control::Jump, 0.94, 0.55, 0.15),
            (Control::Sneak, 0.94, 0.75, 0.15),
        ] { row[control as usize] = Placement::new(x, y, size); }
    }
}
impl Default for Settings {
    fn default() -> Self {
        let base = [
            Placement::new(0.13, 0.76, 0.32),
            Placement::new(0.72, 0.78, 0.105),
            Placement::new(0.92, 0.78, 0.16),
            Placement::new(0.81, 0.86, 0.115),
            Placement::new(0.80, 0.64, 0.16),
            Placement::new(0.92, 0.52, 0.16),
            Placement::new(0.46, 0.055, 0.075),
            Placement::new(0.50, 0.055, 0.075),
            Placement::new(0.54, 0.055, 0.075),
            Placement::new(0.73, 0.95, 0.075),
            Placement::new(0.42, 0.055, 0.075),
            Placement::new(0.58, 0.055, 0.075),
            Placement::new(0.80, 0.96, 0.075),
            Placement::new(0.92, 0.36, 0.12),
        ];
        let mut layouts = [base; 3];
        layouts[2][Control::Sneak as usize] = Placement::new(0.13, 0.76, 0.10);
        layouts[2][Control::Jump as usize] = Placement::new(0.90, 0.78, 0.14);
        #[cfg(target_os = "android")]
        craftix_factory_layout(&mut layouts);
        Self {
            mode: Mode::Crosshair,
            split_controls: false,
            contextual_interact: false,
            left_handed: false,
            sprint_with_joystick: false,
            swap_jump_sneak: false,
            sensitivity: 1.0,
            sensitivity_horizontal: 1.0,
            sensitivity_vertical: 1.0,
            offhand_button: false,
            shield_button: false,
            circle_size: 32.0,
            button_scale: 1.0,
            custom_buttons: Vec::new(),
            analog_movement: false,
            sprint_button: true,
            sprint_toggle: false,
            inventory_controls: false,
            virtual_mouse_sensitivity: 1.0,
            crosshair_gestures: false,
            gyroscope: false,
            gyro_sensitivity: 1.0,
            attack_indicator_left: true,
            button_visibility: [0; 14],
            horizontal_margin: 0.0,
            vertical_margin: 0.0,
            layouts,
        }
    }
}
impl Settings {
    pub fn load() -> Self {
        Self::load_from(std::path::Path::new("horizon-touch.json"))
    }
    pub fn load_from(path: &std::path::Path) -> Self {
        let mut s: Self = std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        s.sanitize();
        // Restore the original top-row spacing without moving custom layouts.
        for row in &mut s.layouts {
            for (control, old, new) in [
                (Control::Chat, 0.45, 0.46),
                (Control::Settings, 0.55, 0.54),
                (Control::Debug, 0.40, 0.42),
                (Control::Perspective, 0.60, 0.58),
            ] {
                let p = &mut row[control as usize];
                if p.center == [old, 0.055] && p.size == 0.075 {
                    p.center[0] = new;
                }
            }
        }
        // Migrate factory Swap placements only; keep the player's own layout.
        for row in &mut s.layouts {
            let swap = &mut row[Control::Offhand as usize];
            if matches!(swap.center, [0.68, 0.80] | [0.66, 0.38])
                && (swap.size - 0.09).abs() < 0.0001
            {
                swap.center = [0.80, 0.96];
                swap.size = 0.075;
            }
        }
        // Move the old factory sprint position only; keep customized placements.
        for mode in [Mode::Crosshair, Mode::Tap] {
            let sprint = &mut s.layouts[mode.index()][Control::Sprint as usize];
            if sprint.center == [0.13, 0.50] && (sprint.size - 0.105).abs() < 0.0001 {
                sprint.center = [0.72, 0.78];
            }
        }
        s
    }
    pub fn sanitize(&mut self) {
        if self.mode == Mode::Tap && self.split_controls {
            self.mode = Mode::Crosshair;
            self.split_controls = false;
            self.crosshair_gestures = true;
        }
        self.virtual_mouse_sensitivity = if self.virtual_mouse_sensitivity.is_finite() {
            self.virtual_mouse_sensitivity.clamp(0.25, 3.0)
        } else {
            1.0
        };
        self.gyro_sensitivity = if self.gyro_sensitivity.is_finite() {
            self.gyro_sensitivity.clamp(0.1, 3.0)
        } else {
            1.0
        };
        for value in [&mut self.horizontal_margin, &mut self.vertical_margin] {
            *value = if value.is_finite() {
                value.clamp(0.0, 0.12)
            } else {
                0.0
            };
        }
        if !self.sensitivity.is_finite() {
            self.sensitivity = 1.0;
        }
        self.sensitivity = self.sensitivity.clamp(0.1, 3.0);
        for value in [
            &mut self.sensitivity_horizontal,
            &mut self.sensitivity_vertical,
        ] {
            *value = if value.is_finite() {
                value.clamp(0.1, 3.0)
            } else {
                1.0
            };
        }
        self.button_scale = if self.button_scale.is_finite() {
            self.button_scale.clamp(0.5, 2.0)
        } else {
            1.0
        };
        self.circle_size = if self.circle_size.is_finite() {
            self.circle_size.clamp(16.0, 64.0)
        } else {
            32.0
        };
        self.custom_buttons.truncate(32);
        self.custom_buttons
            .retain(|b| super::CustomButton::valid_token(&b.key));
        for button in &mut self.custom_buttons {
            button.name = button.name.chars().filter(|c| !c.is_control()).take(32).collect();
            for p in &mut button.layouts {
                p.sanitize();
            }
        }
        for p in self.layouts.iter_mut().flatten() {
            p.sanitize();
        }
    }
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(std::path::Path::new("horizon-touch.json"))
    }
    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(temp, path)
    }
    pub fn crosshair(&self) -> bool {
        self.mode == Mode::Crosshair || self.split_controls
    }
    pub fn visible(&self, c: Control) -> bool {
        match self.button_visibility[c as usize] {
            1 => return true,
            2 => return false,
            _ => {}
        }
        match c {
            Control::Offhand => self.offhand_button,
            Control::Shield => self.shield_button,
            Control::Attack | Control::Use => self.crosshair(),
            Control::Sprint => {
                self.sprint_button && self.mode != Mode::Dpad && !self.sprint_with_joystick
            }
            _ => true,
        }
    }
    pub fn slot(&self, c: Control) -> usize {
        match c {
            Control::Jump if self.mode == Mode::Dpad && self.swap_jump_sneak => {
                Control::Sneak as usize
            }
            Control::Sneak if self.mode == Mode::Dpad && self.swap_jump_sneak => {
                Control::Jump as usize
            }
            _ => c as usize,
        }
    }
    pub fn placement(&self, c: Control) -> Placement {
        let mut p = self.layouts[self.mode.index()][self.slot(c)];
        if self.left_handed {
            p.center[0] = 1.0 - p.center[0];
        }
        p
    }
}
/// A portable, bounded control layout; gameplay and graphics settings are not exported.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutFile {
    schema: u32,
    mode: Mode,
    split_controls: bool,
    left_handed: bool,
    button_scale: f32,
    #[serde(deserialize_with = "deserialize_visibility")]
    button_visibility: [u8; 14],
    #[serde(deserialize_with = "deserialize_layouts")]
    layouts: [[Placement; 14]; 3],
    custom_buttons: Vec<super::CustomButton>,
}
const MAX_LAYOUT_BYTES: usize = 128 * 1024;

impl Settings {
    /// Export only the control layout, not player information or preferences.
    pub fn layout_json(&self) -> std::io::Result<Vec<u8>> {
        let layout = LayoutFile {
            schema: 1, mode: self.mode, split_controls: self.split_controls,
            left_handed: self.left_handed, button_scale: self.button_scale,
            button_visibility: self.button_visibility, layouts: self.layouts,
            custom_buttons: self.custom_buttons.clone(),
        };
        let data = serde_json::to_vec_pretty(&layout)?;
        if data.len() > MAX_LAYOUT_BYTES {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Layout too large"));
        }
        Ok(data)
    }
    /// Validate completely before modifying the active settings.
    pub fn import_layout_json(&mut self, data: &[u8]) -> std::io::Result<()> {
        if data.len() > MAX_LAYOUT_BYTES {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Layout too large"));
        }
        let layout: LayoutFile = serde_json::from_slice(data)?;
        if layout.schema != 1 || layout.custom_buttons.len() > 32
            || layout.button_visibility.iter().any(|v| *v > 2)
            || layout.custom_buttons.iter().any(|b| !super::CustomButton::valid_token(&b.key))
        {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported control layout"));
        }
        let mut next = self.clone();
        next.mode = layout.mode;
        next.split_controls = layout.split_controls;
        next.left_handed = layout.left_handed;
        next.button_scale = layout.button_scale;
        next.button_visibility = layout.button_visibility;
        next.layouts = layout.layouts;
        next.custom_buttons = layout.custom_buttons;
        next.sanitize();
        *self = next;
        Ok(())
    }
    /// Built-in templates only change controls; unrelated preferences survive.
    pub fn apply_control_preset(&mut self, left_handed: bool) {
        let defaults = Self::default();
        self.layouts = defaults.layouts;
        self.button_visibility = defaults.button_visibility;
        self.mode = Mode::Crosshair;
        self.split_controls = false;
        self.left_handed = left_handed;
        self.button_scale = 1.0;
    }
}

pub fn contains([x, y, w, h]: [f32; 4], p: [f32; 2]) -> bool {
    (x..=x + w).contains(&p[0]) && (y..=y + h).contains(&p[1])
}

#[cfg(test)]
mod tests {
    #[test]
    fn craftix_factory_controls_do_not_overlap_on_landscape_phones() {
        let mut layouts = Settings::default().layouts;
        craftix_factory_layout(&mut layouts);
        for aspect in [16.0 / 9.0, 20.0 / 9.0] {
            for mode in [Mode::Crosshair, Mode::Tap] {
                let row = &layouts[mode.index()];
                for (a,b) in [(Control::Move,Control::Sprint), (Control::Attack,Control::Use),
                    (Control::Attack,Control::Jump), (Control::Use,Control::Sneak), (Control::Jump,Control::Sneak)] {
                    let [x,y,w,h] = row[a as usize].rect(aspect);
                    let [u,v,s,t] = row[b as usize].rect(aspect);
                    assert!(x+w <= u || u+s <= x || y+h <= v || v+t <= y, "{mode:?}: {a:?} overlaps {b:?}");
                }
            }
        }
    }
    #[test]
    fn gyro_preferences_roundtrip_and_reject_bad_values() {
        let mut s = super::Settings::default();
        assert!(!s.gyroscope);
        s.gyroscope = true;
        s.gyro_sensitivity = 2.0;
        let restored: super::Settings =
            serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert!(restored.gyroscope);
        assert_eq!(restored.gyro_sensitivity, 2.0);
        s.gyro_sensitivity = f32::NAN;
        s.sanitize();
        assert_eq!(s.gyro_sensitivity, 1.0);
    }
    #[test]
    fn blank_buttons_and_game_actions_survive_settings_validation() {
        let mut settings = super::Settings::default();
        settings
            .custom_buttons
            .push(super::super::CustomButton::new(
                "key.keyboard.unknown".into(),
                0,
            ));
        settings
            .custom_buttons
            .push(super::super::CustomButton::new("action:key.jump".into(), 1));
        settings.sanitize();
        assert_eq!(settings.custom_buttons.len(), 2);
        let encoded = serde_json::to_string(&settings.custom_buttons[1]).unwrap();
        assert_eq!(
            serde_json::from_str::<super::super::CustomButton>(&encoded)
                .unwrap()
                .key,
            "action:key.jump"
        );
        assert!(!super::super::CustomButton::valid_token(
            "action:no-such-action"
        ));
        assert!(
            super::super::custom::choices(false)
                .iter()
                .any(|(token, _)| token == "action:key.attack")
        );
        assert!(
            super::super::custom::choices(true)
                .iter()
                .any(|(token, _)| token == "key.keyboard.z")
        );
    }

    use super::*;

    #[test]
    fn added_buttons_override_scheme_and_removed_buttons_stay_hidden() {
        let mut settings = Settings {
            mode: Mode::Tap,
            ..Settings::default()
        };
        assert!(!settings.visible(Control::Attack));
        settings.button_visibility[Control::Attack as usize] = 1;
        assert!(settings.visible(Control::Attack));
        settings.button_visibility[Control::Chat as usize] = 2;
        assert!(!settings.visible(Control::Chat));
        let restored: Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert!(restored.visible(Control::Attack));
        assert!(!restored.visible(Control::Chat));
    }

    #[test]
    fn old_layout_preserves_custom_buttons_when_shortcuts_are_added() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        json["circle_size"] = 48.into();
        for row in json["layouts"].as_array_mut().unwrap() {
            row.as_array_mut().unwrap().truncate(10);
            row[6]["center"] = serde_json::json!([0.25, 0.2]);
            row[6]["opacity"] = serde_json::json!(0.35);
        }
        let settings: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(settings.circle_size, 48.0);
        for row in settings.layouts {
            assert_eq!(row[6].center, [0.25, 0.2]);
            assert_eq!(row[6].opacity, 0.35);
            assert_eq!(row[Control::Debug as usize].center, [0.42, 0.055]);
            assert_eq!(row[Control::Perspective as usize].center, [0.58, 0.055]);
        }
        let roundtrip: Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert_eq!(roundtrip.layouts[0][6].center, [0.25, 0.2]);
    }
    #[test]
    fn swap_factory_migration_keeps_custom_positions() {
        let file =
            std::env::temp_dir().join(format!("horizon-swap-layout-{}.json", std::process::id()));
        let mut settings = Settings::default();
        settings.layouts[0][Control::Offhand as usize] = super::Placement::new(0.68, 0.80, 0.09);
        settings.layouts[1][Control::Offhand as usize].center = [0.55, 0.43];
        settings.layouts[2][Control::Offhand as usize] = super::Placement::new(0.66, 0.38, 0.09);
        settings.save_to(&file).unwrap();
        let read = Settings::load_from(&file);
        assert_eq!(
            read.layouts[0][Control::Offhand as usize].center,
            [0.80, 0.96]
        );
        assert_eq!(
            read.layouts[1][Control::Offhand as usize].center,
            [0.55, 0.43]
        );
        assert_eq!(
            read.layouts[2][Control::Offhand as usize].center,
            [0.80, 0.96]
        );
        std::fs::remove_file(file).unwrap();
    }
}

#[cfg(test)]
mod portable_layout_tests {
    use super::*;
    #[test]
    fn round_trip_keeps_layout_but_not_other_preferences() {
        let mut source = Settings::default();
        source.left_handed = true;
        source.layouts[0][0].center = [0.3, 0.7];
        let mut button = super::super::CustomButton::new("key.keyboard.z".into(), 0);
        button.name = "Build".into();
        button.icon = super::super::CustomIcon::Attack;
        button.enabled = false;
        button.toggle = true;
        source.custom_buttons.push(button);
        let json = source.layout_json().unwrap();
        let mut target = Settings::default();
        target.sensitivity = 2.0;
        target.import_layout_json(&json).unwrap();
        assert_eq!(target.sensitivity, 2.0);
        assert!(target.left_handed);
        assert_eq!(target.layouts[0][0].center, [0.3, 0.7]);
        assert_eq!(target.custom_buttons[0].name, "Build");
        assert_eq!(target.custom_buttons[0].icon, super::super::CustomIcon::Attack);
        assert!(!target.custom_buttons[0].enabled);
        assert!(target.custom_buttons[0].toggle);
    }
    #[test]
    fn invalid_import_is_transactional_and_bounded() {
        let mut target = Settings::default();
        target.left_handed = true;
        assert!(target.import_layout_json(b"{not json").is_err());
        assert!(target.left_handed);
        assert!(target.import_layout_json(&vec![b' '; MAX_LAYOUT_BYTES + 1]).is_err());
        assert!(target.left_handed);
        let mut json: serde_json::Value = serde_json::from_slice(&target.layout_json().unwrap()).unwrap();
        json["schema"] = 99.into();
        assert!(target.import_layout_json(&serde_json::to_vec(&json).unwrap()).is_err());
        assert!(target.left_handed);
    }
}
