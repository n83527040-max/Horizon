//! Ordered translation from the new router to the game's existing input vocabulary.
//!
//! Android has an experimental opt-in host path using this adapter. The normal
//! game still uses the legacy controls until integration is fully verified.
//! Hosts must share ownership with physical/controller inputs and release on
//! lifecycle changes; UI requests require explicit host handling.
use super::{input::Event, layout::Action};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction { More, Pause, Settings, Keyboard, Group(u16) }

#[derive(Clone, Debug, PartialEq)]
pub enum HostEvent {
    /// Raw `key.keyboard.*` or `action:key.*`, accepted by Android's existing
    /// custom-input dispatch. This still needs to be delivered by the host.
    Input(String, bool),
    Ui(UiAction, bool),
    Look([f32; 2]),
    Move([f32; 2]),
}

/// Different actions may map to the same binding (e.g. builtin Jump and a
/// custom Jump). Aggregate before emitting edges so one release cannot strand
/// or prematurely release the other. Raw keys and bindings can overlap after
/// rebinding; the host must additionally aggregate the resolved engine inputs.
#[derive(Default)]
pub struct Bridge { owners: BTreeMap<String, BTreeSet<Action>>, order: Vec<String> }
impl Bridge {
    pub fn translate(&mut self, events: impl IntoIterator<Item = Event>) -> Vec<HostEvent> {
        let mut out = Vec::new();
        for event in events {
            match event {
                Event::Move(v) => out.push(HostEvent::Move(v)),
                Event::Look(v) => out.push(HostEvent::Look(v)),
                Event::Key(action, down) => {
                    if let Some(ui) = match action {
                        Action::More => Some(UiAction::More),
                        Action::Pause => Some(UiAction::Pause),
                        Action::Settings => Some(UiAction::Settings),
                        Action::Keyboard => Some(UiAction::Keyboard),
                        Action::Group(group) => Some(UiAction::Group(group)),
                        _ => None,
                    } {
                        out.push(HostEvent::Ui(ui, down));
                        continue;
                    }
                    let mut tokens = match &action {
                        Action::Sprint => vec!["action:key.sprint".into()],
                        Action::Jump => vec!["action:key.jump".into()],
                        Action::Sneak => vec!["action:key.sneak".into()],
                        Action::Attack => vec!["action:key.attack".into()],
                        Action::Use => vec!["action:key.use".into()],
                        Action::Inventory => vec!["action:key.inventory".into()],
                        Action::Chat => vec!["action:key.chat".into()],
                        Action::Players => vec!["action:key.playerlist".into()],
                        Action::Perspective => vec!["action:key.togglePerspective".into()],
                        Action::Key(name) if crate::keybinds::key_code(name).is_some() => vec![name.clone()],
                        Action::Chord { modifiers, key } if super::layout::valid_chord(modifiers, key) => {
                            let mut tokens = modifiers.clone(); tokens.push(key.clone()); tokens
                        }
                        Action::Binding(name) if super::layout::dispatchable_binding(name) => vec![format!("action:{name}")],
                        _ => continue,
                    };
                    if !down { tokens.reverse(); }
                    for token in tokens {
                        if down {
                            let owners = self.owners.entry(token.clone()).or_default();
                            let first = owners.is_empty();
                            if owners.insert(action.clone()) && first {
                                self.order.push(token.clone());
                                out.push(HostEvent::Input(token, true));
                            }
                        } else if let Some(owners) = self.owners.get_mut(&token) {
                            if owners.remove(&action) && owners.is_empty() {
                                self.owners.remove(&token);
                                self.order.retain(|held| held != &token);
                                out.push(HostEvent::Input(token, false));
                            }
                        }
                    }
                }
            }
        }
        out
    }
    /// Safety net if the host tears down without draining router releases.
    pub fn clear(&mut self) -> Vec<HostEvent> {
        self.owners.clear();
        std::mem::take(&mut self.order).into_iter().rev().map(|token| HostEvent::Input(token, false)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_builtin_and_custom_binding_have_balanced_edges() {
        let mut bridge = Bridge::default();
        let jump = Action::Jump;
        let custom = Action::Binding("key.jump".into());
        assert_eq!(bridge.translate([Event::Key(jump.clone(), true), Event::Key(custom.clone(), true)]),
            vec![HostEvent::Input("action:key.jump".into(), true)]);
        assert!(bridge.translate([Event::Key(jump, false)]).is_empty());
        assert_eq!(bridge.translate([Event::Key(custom, false)]), vec![HostEvent::Input("action:key.jump".into(), false)]);
    }
    #[test]
    fn raw_m_and_mouse_actions_emit_real_existing_tokens() {
        let mut bridge = Bridge::default();
        let action = Action::Key("key.keyboard.m".into());
        assert_eq!(bridge.translate([Event::Key(action.clone(), true), Event::Key(Action::Attack, true), Event::Key(action, false)]), vec![
            HostEvent::Input("key.keyboard.m".into(),true), HostEvent::Input("action:key.attack".into(),true),
            HostEvent::Input("key.keyboard.m".into(),false)]);
        assert_eq!(bridge.clear(), vec![HostEvent::Input("action:key.attack".into(),false)]);
    }
    #[test]
    fn chords_order_edges_and_share_keys_without_early_release() {
        let mut bridge=Bridge::default();
        let ctrl_m=super::super::layout::parse_key_action("Ctrl + M").unwrap();
        let ctrl_k=super::super::layout::parse_key_action("Ctrl + K").unwrap();
        assert_eq!(bridge.translate([Event::Key(ctrl_m.clone(),true)]), vec![
            HostEvent::Input("key.keyboard.left.control".into(),true), HostEvent::Input("key.keyboard.m".into(),true)]);
        assert_eq!(bridge.translate([Event::Key(ctrl_k.clone(),true)]), vec![HostEvent::Input("key.keyboard.k".into(),true)]);
        assert_eq!(bridge.translate([Event::Key(ctrl_m,false)]), vec![HostEvent::Input("key.keyboard.m".into(),false)]);
        assert_eq!(bridge.translate([Event::Key(ctrl_k,false)]), vec![
            HostEvent::Input("key.keyboard.k".into(),false),HostEvent::Input("key.keyboard.left.control".into(),false)]);
        let shift_a=super::super::layout::parse_key_action("Shift+A").unwrap();
        let alt_k=super::super::layout::parse_key_action("Alt+K").unwrap();
        bridge.translate([Event::Key(shift_a,true),Event::Key(alt_k,true)]);
        assert_eq!(bridge.clear(),vec![HostEvent::Input("key.keyboard.k".into(),false),
            HostEvent::Input("key.keyboard.left.alt".into(),false),HostEvent::Input("key.keyboard.a".into(),false),
            HostEvent::Input("key.keyboard.left.shift".into(),false)]);
    }
    #[test]
    fn ui_and_motion_are_not_fabricated_as_keyboard_events() {
        let mut b = Bridge::default();
        assert_eq!(b.translate([Event::Key(Action::More, true), Event::Look([0.1, 0.2]), Event::Move([0.3, 0.4])]), vec![
            HostEvent::Ui(UiAction::More,true), HostEvent::Look([0.1,0.2]), HostEvent::Move([0.3,0.4])]);
    }
}
