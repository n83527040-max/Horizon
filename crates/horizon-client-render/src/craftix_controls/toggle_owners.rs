//! Merge touch-owned levels with the engine's independently toggled mappings.
//! The caller forwards only effective edges to its keyboard/mouse handlers.
use crate::{keymapping::ToggleKeys, touch::InputOwners};
use winit::{event::MouseButton, keyboard::KeyCode};

type Input = Result<KeyCode, MouseButton>;

fn slot(input: Input) -> Option<usize> {
    match input {
        Ok(KeyCode::ShiftLeft) => Some(0),
        Ok(KeyCode::ControlLeft) => Some(1),
        Err(MouseButton::Left) => Some(2),
        Err(MouseButton::Right) => Some(3),
        _ => None,
    }
}

#[derive(Debug, Default)]
pub struct ToggleOwners(InputOwners<Input>);
impl ToggleOwners {
    /// Tracks resolved inputs until release even if bindings change. `active`
    /// must be false outside gameplay, but releases are always processed.
    pub fn edge(&mut self, token: &str, down: bool, inputs: Vec<Input>, active: bool, toggles: &mut ToggleKeys)
        -> (Vec<Input>, Vec<(Input, bool)>) {
        let (special, other) = inputs.into_iter().partition(|input| slot(*input).is_some());
        let mut edges = Vec::new();
        for (input, pressed) in self.0.edge(token, down, special) {
            let slot = slot(input).expect("only toggle inputs stored");
            let before = toggles.is_down(slot);
            toggles.virtual_hold(slot, pressed && active);
            let after = toggles.is_down(slot);
            if before != after { edges.push((input, after)); }
        }
        (other, edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{craftix_controls::{bridge::{Bridge, HostEvent}, input::Router, layout::{Action, Behavior, Button, Circle, Icon, Id, Layout, Visibility}}, keybinds::KeyBinds, keymapping::{KeyAction, SNEAK}};

    #[test]
    fn router_bridge_and_real_toggle_ownership_preserve_hardware_and_other_fingers() {
        let mut layout=Layout::default();
        let builtin=layout.buttons.iter_mut().find(|b| b.id==Id::Sneak).unwrap();
        builtin.circle.center=[0.60,0.25]; builtin.behavior=Behavior::Hold;
        layout.buttons.push(Button { id: Id::Custom(47), action: Some(Action::Key("key.keyboard.k".into())),
            circle: Circle { center: [0.75,0.25], radius: 0.04 }, opacity: 1.0,
            behavior: Behavior::Hold, icon: Icon::Text, enabled: true, visibility: Visibility::Always, label: "K".into() });
        let mut bindings=KeyBinds::default(); bindings.set("key.sneak","key.keyboard.k");
        let mut router=Router::default(); let mut bridge=Bridge::default();
        let mut owners=ToggleOwners::default(); let mut toggles=ToggleKeys::default();
        let pump=|router: &mut Router, bridge: &mut Bridge, owners: &mut ToggleOwners, toggles: &mut ToggleKeys| {
            let mut result=Vec::new();
            for event in bridge.translate(router.drain()) {
                if let HostEvent::Input(token, down)=event {
                    let inputs=if token=="action:key.sneak" { vec![Ok(KeyCode::ShiftLeft)] }
                        else { bindings.translate_all(crate::keybinds::key_code(&token).unwrap()) };
                    let (other, edges)=owners.edge(&token,down,inputs,true,toggles);
                    assert!(other.is_empty()); result.extend(edges);
                }
            }
            result
        };
        toggles.key(SNEAK,KeyAction::Press,false,true); toggles.key(SNEAK,KeyAction::Release,false,true);
        router.begin(&layout,1,[0.60,0.25],16.0/9.0,0); assert!(pump(&mut router,&mut bridge,&mut owners,&mut toggles).is_empty());
        router.begin(&layout,2,[0.75,0.25],16.0/9.0,0); assert!(pump(&mut router,&mut bridge,&mut owners,&mut toggles).is_empty());
        toggles.key(SNEAK,KeyAction::Press,false,true); toggles.key(SNEAK,KeyAction::Release,false,true);
        assert!(toggles.is_down(SNEAK));
        router.end(1); assert!(pump(&mut router,&mut bridge,&mut owners,&mut toggles).is_empty());
        assert!(toggles.is_down(SNEAK));
        router.clear();
        assert_eq!(pump(&mut router,&mut bridge,&mut owners,&mut toggles),vec![(Ok(KeyCode::ShiftLeft),false)]);
        assert!(!toggles.is_down(SNEAK));
    }
    #[test]
    fn modes_emit_balanced_levels_even_when_engine_toggle_is_enabled() {
        for mode in [Behavior::Single, Behavior::Hold, Behavior::Toggle] {
            let mut layout=Layout::default();
            let button=layout.buttons.iter_mut().find(|b| b.id==Id::Sneak).unwrap();
            button.behavior=mode; let point=button.circle.center;
            let mut router=Router::default(); let mut bridge=Bridge::default();
            let mut owners=ToggleOwners::default(); let mut toggles=ToggleKeys::default();
            let pump=|router: &mut Router, bridge: &mut Bridge, owners: &mut ToggleOwners, toggles: &mut ToggleKeys| {
                let mut result=Vec::new();
                for event in bridge.translate(router.drain()) {
                    if let HostEvent::Input(token,down)=event {
                        let (_,edges)=owners.edge(&token,down,vec![Ok(KeyCode::ShiftLeft)],true,toggles);
                        result.extend(edges);
                    }
                }
                result
            };
            router.begin(&layout,1,point,16.0/9.0,0);
            assert_eq!(pump(&mut router,&mut bridge,&mut owners,&mut toggles),vec![(Ok(KeyCode::ShiftLeft),true)]);
            assert!(toggles.is_down(SNEAK));
            router.end(1);
            if mode==Behavior::Single { router.tick(100); }
            if mode==Behavior::Toggle { router.begin(&layout,2,point,16.0/9.0,110); router.end(2); }
            assert_eq!(pump(&mut router,&mut bridge,&mut owners,&mut toggles),vec![(Ok(KeyCode::ShiftLeft),false)]);
            assert!(!toggles.is_down(SNEAK));
        }
    }

}
