//! Ordered translation from the new router to the game's existing input vocabulary.
//!
//! This is an adapter, not an active game integration. The host must dispatch
//! each input edge through the same ownership logic as physical/controller
//! input, handle lifecycle releases, and implement the UI requests explicitly.
use super::{input::Event, layout::Action};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction { More, Pause, Settings }

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
pub struct Bridge { owners: BTreeMap<String, BTreeSet<Action>> }
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
                        _ => None,
                    } {
                        out.push(HostEvent::Ui(ui, down));
                        continue;
                    }
                    let token = match &action {
                        Action::Sprint => "action:key.sprint".into(),
                        Action::Jump => "action:key.jump".into(),
                        Action::Sneak => "action:key.sneak".into(),
                        Action::Attack => "action:key.attack".into(),
                        Action::Use => "action:key.use".into(),
                        Action::Inventory => "action:key.inventory".into(),
                        Action::Chat => "action:key.chat".into(),
                        Action::Players => "action:key.playerlist".into(),
                        Action::Perspective => "action:key.togglePerspective".into(),
                        Action::Key(name) if crate::keybinds::key_code(name).is_some() => name.clone(),
                        Action::Binding(name) if super::layout::dispatchable_binding(name) => format!("action:{name}"),
                        _ => continue,
                    };
                    if down {
                        let owners = self.owners.entry(token.clone()).or_default();
                        let first = owners.is_empty();
                        if owners.insert(action) && first { out.push(HostEvent::Input(token, true)); }
                    } else if let Some(owners) = self.owners.get_mut(&token) {
                        if owners.remove(&action) && owners.is_empty() {
                            self.owners.remove(&token);
                            out.push(HostEvent::Input(token, false));
                        }
                    }
                }
            }
        }
        out
    }
    /// Safety net if the host tears down without draining router releases.
    pub fn clear(&mut self) -> Vec<HostEvent> {
        std::mem::take(&mut self.owners).into_keys().map(|token| HostEvent::Input(token, false)).collect()
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
    fn ui_and_motion_are_not_fabricated_as_keyboard_events() {
        let mut b = Bridge::default();
        assert_eq!(b.translate([Event::Key(Action::More, true), Event::Look([0.1, 0.2]), Event::Move([0.3, 0.4])]), vec![
            HostEvent::Ui(UiAction::More,true), HostEvent::Look([0.1,0.2]), HostEvent::Move([0.3,0.4])]);
    }
}
