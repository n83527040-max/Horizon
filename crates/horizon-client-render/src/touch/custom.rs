//! User-added keys share drawing/hit geometry and retain per-finger ownership.
use super::*;
use layout::Placement;
use serde::{Deserialize, Serialize};
/// Icon is presentation only: the bound engine action/key remains authoritative.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CustomIcon {
    #[default]
    Text,
    Attack,
    Use,
    Jump,
    Sneak,
    Sprint,
}
impl CustomIcon {
    pub fn next(self) -> Self {
        match self {
            Self::Text => Self::Attack,
            Self::Attack => Self::Use,
            Self::Use => Self::Jump,
            Self::Jump => Self::Sneak,
            Self::Sneak => Self::Sprint,
            Self::Sprint => Self::Text,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text", Self::Attack => "Sword", Self::Use => "Use",
            Self::Jump => "Jump", Self::Sneak => "Sneak", Self::Sprint => "Sprint",
        }
    }
    fn sprite(self) -> Option<&'static str> {
        match self {
            Self::Text => None, Self::Attack => Some("ui/attack"),
            Self::Use => Some("ui/interact"), Self::Jump => Some("ui/jump"),
            Self::Sneak => Some("ui/sneak"), Self::Sprint => Some("ui/sprint"),
        }
    }
}
/// Contexts in which a configured key is eligible to receive touch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CustomVisibility { #[default] Always, Gameplay, Inventory }
impl CustomVisibility {
    pub fn next(self) -> Self {
        match self { Self::Always => Self::Gameplay, Self::Gameplay => Self::Inventory, Self::Inventory => Self::Always }
    }
    pub fn label(self) -> &'static str {
        match self { Self::Always => "Always", Self::Gameplay => "Gameplay", Self::Inventory => "Inventory" }
    }
    fn permits(self, inventory: bool) -> bool {
        match self { Self::Always => true, Self::Gameplay => !inventory, Self::Inventory => inventory }
    }
}
fn enabled_by_default() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomButton {
    pub key: String,
    pub layouts: [Placement; 3],
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub icon: CustomIcon,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    #[serde(default)]
    pub visibility: CustomVisibility,
    /// Toggle keeps the key down between taps; off by default for old layouts.
    #[serde(default)]
    pub toggle: bool,
}
impl CustomButton {
    pub fn new(key: String, index: usize) -> Self {
        Self {
            key,
            name: String::new(),
            icon: CustomIcon::Text,
            enabled: true,
            visibility: CustomVisibility::Always,
            toggle: false,
            layouts: [Placement::new(
                0.68 + (index % 3) as f32 * 0.1,
                0.35 + (index / 3 % 3) as f32 * 0.12,
                0.105,
            ); 3],
        }
    }
    pub fn valid_token(token: &str) -> bool {
        token == "key.keyboard.unknown"
            || crate::keybinds::key_code(token).is_some()
            || token
                .strip_prefix("action:")
                .is_some_and(|name| crate::keybinds::MAPPINGS.iter().any(|m| m.0 == name))
    }
    pub fn label(&self) -> String {
        if !self.name.trim().is_empty() {
            return self.name.clone();
        }
        self.binding_label()
    }
    pub fn binding_label(&self) -> String {
        if self.key == "key.keyboard.unknown" {
            return "Button".into();
        }
        if let Some(name) = self.key.strip_prefix("action:") {
            return crate::text::Text::translate(name, &[]).plain();
        }
        crate::keybinds::display_name(&self.key).plain()
    }
}
pub(super) fn choices(raw_keys: bool) -> Vec<(String, String)> {
    if raw_keys {
        crate::keybinds::available_keys()
            .iter()
            .map(|(name, _)| ((*name).into(), crate::keybinds::display_name(name).plain()))
            .collect()
    } else {
        crate::keybinds::MAPPINGS
            .iter()
            .filter(|m| m.1 != "key.keyboard.unknown")
            .map(|(name, _, _)| {
                (
                    format!("action:{name}"),
                    crate::text::Text::translate(name, &[]).plain(),
                )
            })
            .collect()
    }
}
impl TouchControls {
    pub(super) fn inventory_token(control: Control) -> Option<&'static str> {
        match control {
            Control::Sneak => Some("key.keyboard.left.shift"),
            Control::Attack => Some("action:key.attack"),
            Control::Use => Some("action:key.use"),
            _ => None,
        }
    }
    pub fn inventory_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if self.custom_fingers.contains_key(&id) || self.toggle_fingers.contains_key(&id) {
            return self.custom_update(id, point, false, ended);
        }
        if started
            && self.custom_at(point).is_none()
            && let Some(token) = self.control_at(point).and_then(Self::inventory_token)
        {
            let pressed = !self.custom_token_active(token);
            self.custom_fingers.insert(id, token.into());
            self.fingers.insert(id, (Role::Ignore, point));
            if pressed {
                self.custom_events.push((token.into(), true));
            }
            return true;
        }
        false
    }
    pub(super) fn fit_placement(&self, mut p: Placement) -> Placement {
        p.size = (p.size * self.settings.button_scale)
            .min(self.aspect())
            .min(1.0);
        let half = [p.size / self.aspect() / 2.0, p.size / 2.0];
        for (i, half) in half.into_iter().enumerate() {
            p.center[i] = p.center[i].clamp(half, 1.0 - half);
        }
        p
    }
    pub(super) fn custom_placement(&self, index: usize) -> Placement {
        let mut p = self.settings.custom_buttons[index].layouts[self.settings.mode.index()];
        if self.settings.left_handed {
            p.center[0] = 1.0 - p.center[0];
        }
        self.fit_placement(p)
    }
    pub fn custom_at(&self, point: [f32; 2]) -> Option<usize> {
        self.custom_at_with_placeholders(point, false)
    }
    pub(super) fn custom_at_with_placeholders(&self, point: [f32; 2], include: bool) -> Option<usize> {
        (0..self.settings.custom_buttons.len()).rev().find(|&i| {
            (include || (self.settings.custom_buttons[i].enabled
                && self.settings.custom_buttons[i].key != "key.keyboard.unknown"
                && self.settings.custom_buttons[i].visibility.permits(self.inventory_mode)
                && self.custom_placement(i).opacity > 0.0))
                && layout::contains(self.custom_placement(i).rect(self.aspect()), point)
        })
    }
    /// Begin an engine-mapped hold without inventing a key event on release.
    pub fn begin_mapped_touch(&mut self, id: u64, token: &str, point: [f32; 2]) -> bool {
        if !CustomButton::valid_token(token) || self.fingers.contains_key(&id) { return false; }
        let first = !self.custom_token_active(token);
        self.custom_fingers.insert(id, token.to_owned());
        self.fingers.insert(id, (Role::Ignore, point));
        if first { self.custom_events.push((token.to_owned(), true)); }
        true
    }
    pub(super) fn custom_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if self.toggle_fingers.contains_key(&id) {
            if ended {
                self.toggle_fingers.remove(&id);
                self.fingers.remove(&id);
            }
            return true;
        }
        if let Some(key) = self.custom_fingers.get(&id).cloned() {
            if ended {
                self.custom_fingers.remove(&id);
                self.fingers.remove(&id);
                if !self.custom_token_active(&key) {
                    self.custom_events.push((key, false));
                }
            }
            return true;
        }
        if started && let Some(index) = self.custom_at(point) {
            let key = self.settings.custom_buttons[index].key.clone();
            if key == "key.keyboard.unknown" {
                return false;
            }
            let active_before = self.custom_token_active(&key);
            if self.settings.custom_buttons[index].toggle {
                // Multiple fingers on the same toggle count as a single tap.
                let first = !self.toggle_fingers.values().any(|v| *v == key);
                self.toggle_fingers.insert(id, key.clone());
                if first {
                    if !self.latched_custom.remove(&key) {
                        self.latched_custom.insert(key.clone());
                    }
                }
            } else {
                self.custom_fingers.insert(id, key.clone());
            }
            self.fingers.insert(id, (Role::Ignore, point));
            let active_after = self.custom_token_active(&key);
            if active_before != active_after {
                self.custom_events.push((key, active_after));
            }
            self.clicked = true;
            return true;
        }
        false
    }
    fn custom_token_active(&self, token: &str) -> bool {
        self.latched_custom.contains(token) || self.custom_fingers.values().any(|k| k == token)
    }
    pub fn held_custom_tokens(&self) -> impl Iterator<Item = &str> {
        self.custom_fingers.values().chain(self.latched_custom.iter()).map(String::as_str)
    }
    pub fn key_events(&mut self) -> Vec<(String, bool)> {
        std::mem::take(&mut self.custom_events)
    }
    pub(super) fn clear_custom(&mut self) {
        let mut keys: std::collections::BTreeSet<_> = std::mem::take(&mut self.custom_fingers)
            .into_values()
            .collect();
        keys.append(&mut self.latched_custom);
        self.toggle_fingers.clear();
        self.custom_events
            .extend(keys.into_iter().map(|k| (k, false)));
    }
    pub(super) fn draw_custom(&self, b: &mut HudBatch, a: &HudAtlas) {
        for (i, button) in self.settings.custom_buttons.iter().enumerate() {
            // Unassigned placeholders are useful in the editor only; outside
            // it they neither pretend to work nor block camera gestures.
            if self.editor.is_none() && (!button.enabled || button.key == "key.keyboard.unknown"
                || !button.visibility.permits(self.inventory_mode)) {
                continue;
            }
            let p = self.custom_placement(i);
            let size = p.size * b.height;
            let rect = [
                p.center[0] * b.width - size / 2.0,
                p.center[1] * b.height - size / 2.0,
                size,
                size,
            ];
            let pressed = self.custom_token_active(&button.key);
            let opacity = if self.editor.is_some() {
                p.opacity.max(0.25)
            } else {
                p.opacity
            };
            if opacity <= 0.0 {
                continue;
            }
            #[cfg(target_os = "android")]
            {
                let start = b.vertices.len();
                let normalized = [rect[0]/b.width, rect[1]/b.height, rect[2]/b.width, rect[3]/b.height];
                editor::glass(b, a, normalized, pressed);
                let icon = button.icon.sprite();
                if let Some(icon) = icon {
                    fitted_sprite(b, a, icon, [rect[0]+size*0.28, rect[1]+size*0.10, size*0.44, size*0.44], rgba(245,232,255,255));
                }
                let label = button.label();
                let width = HudBatch::text_width(a, &label);
                let text_start = b.vertices.len();
                let ty = if icon.is_some() { rect[1]+size*0.68 } else { rect[1]+(size-8.0)/2.0 };
                b.text(a, &label, rect[0]+(size-width)/2.0, ty, rgba(250,242,255,255), false);
                let scale = ((size-6.0).max(1.0)/width.max(1.0)).min(1.0);
                b.scale_since(text_start, rect[0]+size/2.0, ty+4.0, scale, scale);
                if pressed { b.scale_since(start, rect[0]+size/2.0, rect[1]+size/2.0, 0.95, 0.95); }
                for vertex in &mut b.vertices[start..] {
                    let alpha = ((vertex.color >> 24) as f32 * opacity.clamp(0.0,1.0)).round() as u32;
                    vertex.color = (vertex.color & 0x00ff_ffff) | (alpha << 24);
                }
                continue;
            }
            #[cfg(not(target_os = "android"))]
            let color = rgba(255, 255, 255, (opacity * 255.0) as u8);
            #[cfg(not(target_os = "android"))]
            if let Some(icon) = button.icon.sprite() {
                text_button(b, a, rect, "", pressed, color, true);
                fitted_sprite(b, a, icon,
                    [rect[0] + size * 0.25, rect[1] + size * 0.08, size * 0.5, size * 0.5], color);
                let label = button.label();
                let width = HudBatch::text_width(a, &label);
                let start = b.vertices.len();
                b.text(a, &label, rect[0] + (size - width) / 2.0,
                    rect[1] + size * 0.7, color, false);
                let scale = ((size - 4.0).max(1.0) / width.max(1.0)).min(1.0);
                b.scale_since(start, rect[0] + size / 2.0, rect[1] + size * 0.74, scale, scale);
            } else {
                text_button(b, a, rect, &button.label(), pressed, color, true);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_modifier_and_mouse_hold_independently_and_cancel_cleanly() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.inventory_mode = true;
        t.settings.mode = Mode::Crosshair;
        for (id, control, token) in [
            (1, Control::Sneak, "key.keyboard.left.shift"),
            (2, Control::Attack, "action:key.attack"),
            (3, Control::Use, "action:key.use"),
        ] {
            assert!(t.visible(control));
            assert!(t.inventory_update(id, t.placement(control).center, true, false));
            assert_eq!(t.key_events(), [(token.into(), true)]);
        }
        t.clear_world_gesture();
        assert_eq!(t.held_custom_tokens().count(), 3);
        assert_eq!(t.poll_state(), PadState::default());
        assert!(t.inventory_update(2, [0.0, 0.0], false, true));
        assert_eq!(t.key_events(), [("action:key.attack".into(), false)]);
        t.clear();
        let released = t.key_events();
        assert!(released.contains(&("key.keyboard.left.shift".into(), false)));
        assert!(released.contains(&("action:key.use".into(), false)));
    }
    #[test]
    fn inventory_overlay_preserves_configured_visibility_and_placements() {
        let mut touch = TouchControls::default();
        for mode in Mode::ALL {
            touch.settings.mode = mode;
            touch.settings.button_visibility[Control::Attack as usize] = 2;
            let before: Vec<_> = Control::ALL
                .into_iter()
                .map(|c| (c, touch.visible(c), touch.placement(c).center))
                .collect();
            touch.inventory_mode = true;
            for (control, visible, center) in before {
                assert_eq!(touch.visible(control), visible);
                assert_eq!(touch.placement(control).center, center);
            }
            touch.inventory_mode = false;
        }
    }
    #[test]
    fn disabled_button_does_not_capture_or_send_keys() {
        let mut touch = TouchControls::default();
        touch.viewport(1920, 1080);
        let mut button = CustomButton::new("key.keyboard.z".into(), 0);
        button.enabled = false;
        touch.settings.custom_buttons.push(button);
        let point = touch.custom_placement(0).center;
        assert_eq!(touch.custom_at(point), None);
        touch.update(1, point, true, false);
        assert!(touch.key_events().is_empty());
        assert_eq!(touch.custom_at_with_placeholders(point, true), Some(0));
    }
    #[test]
    fn toggle_sends_edges_on_tap_and_clears_on_focus_loss() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        let mut button = CustomButton::new("key.keyboard.m".into(), 0);
        button.toggle = true;
        t.settings.custom_buttons.push(button);
        let p = t.custom_placement(0).center;
        t.update(1, p, true, false);
        t.update(2, p, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), true)]);
        t.update(1, p, false, true);
        t.update(2, p, false, true);
        assert!(t.key_events().is_empty());
        assert_eq!(t.held_custom_tokens().collect::<Vec<_>>(), ["key.keyboard.m"]);
        t.update(3, p, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), false)]);
        t.update(3, p, false, true);
        t.update(4, p, true, false);
        t.update(4, p, false, true);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), true)]);
        t.clear();
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), false)]);
        assert_eq!(t.held_custom_tokens().count(), 0);
    }
    #[test]
    fn toggle_and_hold_share_the_same_input_without_early_release() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        let mut toggle = CustomButton::new("key.keyboard.m".into(), 0);
        toggle.toggle = true;
        t.settings.custom_buttons.push(toggle);
        t.settings.custom_buttons.push(CustomButton::new("key.keyboard.m".into(), 1));
        let p = t.custom_placement(0).center;
        let q = t.custom_placement(1).center;
        t.update(1, p, true, false);
        t.update(1, p, false, true);
        t.update(2, q, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), true)]);
        t.update(3, p, true, false);
        t.update(3, p, false, true);
        assert!(t.key_events().is_empty());
        t.update(2, q, false, true);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), false)]);
    }
    #[test]
    fn context_change_and_resize_release_latched_keys() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        let mut button = CustomButton::new("key.keyboard.m".into(), 0);
        button.toggle = true;
        t.settings.custom_buttons.push(button);
        let p = t.custom_placement(0).center;
        t.update(1, p, true, true);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), true)]);
        t.viewport(1080, 1920);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), false)]);
        t.update(2, t.custom_placement(0).center, true, true);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), true)]);
        t.set_inventory_mode(true);
        assert_eq!(t.key_events(), [("key.keyboard.m".into(), false)]);
    }
    #[test]
    fn virtual_player_list_is_a_reference_counted_hold() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        assert!(t.begin_mapped_touch(1, "action:key.playerlist", [0.62, 0.05]));
        assert!(t.begin_mapped_touch(2, "action:key.playerlist", [0.62, 0.05]));
        assert_eq!(t.key_events(), [("action:key.playerlist".into(), true)]);
        t.update(1, [0.01, 0.01], false, true);
        assert!(t.key_events().is_empty());
        t.update(2, [0.01, 0.01], false, true);
        assert_eq!(t.key_events(), [("action:key.playerlist".into(), false)]);
    }
    #[test]
    fn contextual_visibility_controls_hit_testing() {
        let mut touch = TouchControls::default();
        touch.viewport(1920, 1080);
        let mut button = CustomButton::new("key.keyboard.m".into(), 0);
        button.visibility = CustomVisibility::Inventory;
        touch.settings.custom_buttons.push(button);
        let point = touch.custom_placement(0).center;
        assert_eq!(touch.custom_at(point), None);
        touch.inventory_mode = true;
        assert_eq!(touch.custom_at(point), Some(0));
        touch.update(1, point, true, false);
        touch.update(1, point, false, true);
        assert_eq!(touch.key_events(), [("key.keyboard.m".into(), true), ("key.keyboard.m".into(), false)]);
        touch.settings.custom_buttons[0].layouts[0].opacity = 0.0;
        assert_eq!(touch.custom_at(point), None);
    }
    #[test]
    fn legacy_button_defaults_to_enabled_text() {
        let b = CustomButton::new("key.keyboard.z".into(), 0);
        let mut value = serde_json::to_value(b).unwrap();
        value.as_object_mut().unwrap().remove("enabled");
        value.as_object_mut().unwrap().remove("name");
        value.as_object_mut().unwrap().remove("icon");
        value.as_object_mut().unwrap().remove("visibility");
        value.as_object_mut().unwrap().remove("toggle");
        let loaded: CustomButton = serde_json::from_value(value).unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.icon, CustomIcon::Text);
        assert_eq!(loaded.visibility, CustomVisibility::Always);
        assert!(!loaded.toggle);
        assert!(loaded.name.is_empty());
    }
    #[test]
    fn unassigned_button_emits_no_input() {
        let mut touch = TouchControls::default();
        touch.viewport(1920, 1080);
        touch
            .settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.unknown".into(), 0));
        let point = touch.custom_placement(0).center;
        assert_eq!(touch.custom_at(point), None);
        touch.update(1, point, true, false);
        assert!(touch.key_events().is_empty());
        touch.clear();
        touch.open_editor();
        assert_eq!(touch.custom_at_with_placeholders(point, true), Some(0));
    }
    #[test]
    fn custom_keys_hold_through_motion_and_shared_keys_release_only_after_last_finger() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        let p = t.custom_placement(0).center;
        t.update(1, p, true, false);
        t.update(2, p, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), true)]);
        t.update(1, [0.99, 0.1], false, false);
        assert!(t.key_events().is_empty());
        t.update(1, p, false, true);
        assert!(t.key_events().is_empty());
        t.update(2, p, false, true);
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), false)]);
    }
    #[test]
    fn cancellation_releases_all_custom_keys_and_scaling_preserves_saved_sizes() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        let stored = t.settings.custom_buttons[0].layouts[0].size;
        t.settings.button_scale = 2.0;
        assert_eq!(t.custom_placement(0).size, stored * 2.0);
        t.update(1, t.custom_placement(0).center, true, false);
        t.key_events();
        t.clear();
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), false)]);
        assert_eq!(t.settings.custom_buttons[0].layouts[0].size, stored);
    }
}
