//! Per-pointer ownership and ordered engine edges for the new controls.
use super::layout::{Action, Behavior, Id, Layout};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Event { Key(Action, bool), Look([f32; 2]), Move([f32; 2]) }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Owner { Finger(u64), Toggle(Id), Single(u64) }
#[derive(Clone, Debug)]
enum Role { Joystick { center: [f32; 2], radius: f32 }, Look([f32; 2]), Button, Toggle(Id), Ignore }
#[derive(Default)]
pub struct Router {
    pointers: BTreeMap<u64, Role>,
    owners: BTreeMap<Action, BTreeSet<Owner>>,
    toggles: BTreeMap<Id, Action>,
    singles: BTreeMap<u64, (Action, u64)>,
    next_single: u64,
    joystick: Option<u64>,
    movement: [f32; 2],
    events: Vec<Event>,
}
impl Router {
    fn acquire(&mut self, action: Action, owner: Owner) {
        let owners = self.owners.entry(action.clone()).or_default();
        let first = owners.is_empty();
        owners.insert(owner);
        if first { self.events.push(Event::Key(action, true)); }
    }
    fn release(&mut self, action: &Action, owner: Owner) {
        if let Some(owners) = self.owners.get_mut(action) {
            if !owners.remove(&owner) { return; }
            if owners.is_empty() {
                self.owners.remove(action);
                self.events.push(Event::Key(action.clone(), false));
            }
        }
    }
    /// Monotonic milliseconds, supplied by the host so tests do not sleep.
    pub fn begin(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32, now_ms: u64) {
        if self.pointers.contains_key(&id) || !p.into_iter().all(f32::is_finite) || !aspect.is_finite() || aspect <= 0.0 { return; }
        let role = match layout.hit(p, aspect) {
            Some(button) if button.id == Id::Joystick => {
                if self.joystick.is_some() { Role::Ignore }
                else { self.joystick = Some(id); Role::Joystick { center: button.circle.center, radius: button.circle.radius } }
            }
            Some(button) => {
                if button.behavior == Behavior::Toggle && self.pointers.values().any(|role| matches!(role, Role::Toggle(id) if *id == button.id)) {
                    self.pointers.insert(id, Role::Ignore);
                    return;
                }
                if let Some(action) = &button.action {
                    match button.behavior {
                        Behavior::Hold => self.acquire(action.clone(), Owner::Finger(id)),
                        Behavior::Toggle => {
                            if let Some(old) = self.toggles.remove(&button.id) {
                                self.release(&old, Owner::Toggle(button.id));
                            } else {
                                self.toggles.insert(button.id, action.clone());
                                self.acquire(action.clone(), Owner::Toggle(button.id));
                            }
                        }
                        Behavior::Single => {
                            let single = self.next_single;
                            self.next_single = self.next_single.wrapping_add(1);
                            self.acquire(action.clone(), Owner::Single(single));
                            self.singles.insert(single, (action.clone(), now_ms.saturating_add(100)));
                        }
                    }
                }
                if button.behavior == Behavior::Toggle { Role::Toggle(button.id) } else { Role::Button }
            }
            None => Role::Look(p),
        };
        self.pointers.insert(id, role);
        if self.joystick == Some(id) { self.moved(layout, id, p, aspect); }
    }
    pub fn moved(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32) {
        if !p.into_iter().all(f32::is_finite) || !aspect.is_finite() || aspect <= 0.0 { return; }
        match self.pointers.get_mut(&id) {
            Some(Role::Joystick { center, radius }) => {
                let v = [(p[0] - center[0]) * aspect / *radius, (center[1] - p[1]) / *radius];
                let len = v[0].hypot(v[1]);
                let dead = layout.joystick_dead_zone.clamp(0.0, 0.5);
                let amount = ((len - dead) / (1.0 - dead) * layout.joystick_sensitivity).clamp(0.0, 1.0);
                self.movement = if len > 0.0 { [v[0] / len * amount, v[1] / len * amount] } else { [0.0; 2] };
                self.events.push(Event::Move(self.movement));
            }
            Some(Role::Look(last)) => {
                let delta = [(p[0] - last[0]) * aspect * layout.camera_sensitivity, (p[1] - last[1]) * layout.camera_sensitivity];
                *last = p;
                self.events.push(Event::Look(delta));
            }
            _ => {}
        }
    }
    pub fn end(&mut self, id: u64) {
        if matches!(self.pointers.remove(&id), Some(Role::Joystick { .. })) {
            self.joystick = None;
            self.movement = [0.0; 2];
            self.events.push(Event::Move(self.movement));
        }
        let held: Vec<_> = self.owners.iter().filter(|(_, owners)| owners.contains(&Owner::Finger(id)))
            .map(|(key, _)| key.clone()).collect();
        for key in held { self.release(&key, Owner::Finger(id)); }
    }
    pub fn tick(&mut self, now_ms: u64) {
        let ready: Vec<_> = self.singles.iter().filter(|(_, (_, deadline))| now_ms >= *deadline)
            .map(|(id, _)| *id).collect();
        for id in ready {
            if let Some((action, _)) = self.singles.remove(&id) { self.release(&action, Owner::Single(id)); }
        }
    }
    /// Call for focus loss, a context change, viewport change or shutdown.
    pub fn clear(&mut self) {
        for action in std::mem::take(&mut self.owners).into_keys() { self.events.push(Event::Key(action, false)); }
        self.pointers.clear(); self.toggles.clear(); self.singles.clear(); self.joystick = None;
        if self.movement != [0.0; 2] { self.movement = [0.0; 2]; self.events.push(Event::Move(self.movement)); }
    }
    pub fn movement(&self) -> [f32; 2] { self.movement }
    pub fn active(&self, action: &Action) -> bool { self.owners.contains_key(action) }
    pub fn drain(&mut self) -> Vec<Event> { std::mem::take(&mut self.events) }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::craftix_controls::layout::{Button, Circle};
    fn center(l: &Layout, id: Id) -> [f32; 2] { l.buttons.iter().find(|b| b.id == id).unwrap().circle.center }
    #[test]
    fn joystick_jump_attack_and_camera_own_separate_pointers() {
        let l = Layout::default(); let mut r = Router::default(); let aspect = 16.0/9.0;
        let joy = center(&l, Id::Joystick); r.begin(&l,1,joy,aspect,0);
        r.moved(&l,1,[joy[0], joy[1]-0.1],aspect);
        r.begin(&l,2,center(&l,Id::Jump),aspect,0);
        r.begin(&l,3,center(&l,Id::Attack),aspect,0);
        r.begin(&l,4,[0.5,0.5],aspect,0); r.moved(&l,4,[0.55,0.52],aspect);
        let e = r.drain();
        assert!(e.contains(&Event::Key(Action::Jump,true)) && e.contains(&Event::Key(Action::Attack,true)));
        assert!(e.iter().any(|e| matches!(e, Event::Look(_))));
        assert!(r.movement()[1] > 0.0);
        r.end(2); assert!(r.active(&Action::Attack));
        r.clear(); assert_eq!(r.movement(),[0.0,0.0]);
        assert!(r.drain().contains(&Event::Key(Action::Attack,false)));
    }
    #[test]
    fn two_different_buttons_share_one_action_without_premature_release() {
        let mut l = Layout::default();
        for i in 0..2 { l.buttons.push(Button { id: Id::Custom(i), action: Some(Action::Key("key.keyboard.m".into())),
            circle: Circle { center: [0.65 + i as f32 * 0.1, 0.25], radius: 0.04 }, opacity: 1.0,
            behavior: Behavior::Hold, icon: super::super::layout::Icon::Map, enabled: true, label: "Map".into() }); }
        let mut r = Router::default();
        r.begin(&l,1,center(&l,Id::Custom(0)),16.0/9.0,0);
        r.begin(&l,2,center(&l,Id::Custom(1)),16.0/9.0,0);
        assert_eq!(r.drain(), [Event::Key(Action::Key("key.keyboard.m".into()),true)]);
        r.end(1); assert!(r.drain().is_empty()); r.end(2);
        assert_eq!(r.drain(), [Event::Key(Action::Key("key.keyboard.m".into()),false)]);
    }
    #[test]
    fn reused_pointer_id_does_not_orphan_timed_single_press() {
        let l = Layout::default(); let mut r = Router::default();
        r.begin(&l,1,center(&l,Id::Inventory),16.0/9.0,0); r.end(1);
        r.begin(&l,1,center(&l,Id::More),16.0/9.0,1); r.end(1);
        r.tick(101);
        assert!(!r.active(&Action::Inventory) && !r.active(&Action::More));
    }
    #[test]
    fn two_fingers_on_one_toggle_do_not_flip_it_twice() {
        let mut l=Layout::default(); l.buttons.iter_mut().find(|b| b.id==Id::Sneak).unwrap().behavior=Behavior::Toggle;
        let mut r=Router::default(); let p=center(&l,Id::Sneak);
        r.begin(&l,1,p,16.0/9.0,0); r.begin(&l,2,p,16.0/9.0,0);
        assert_eq!(r.drain(), [Event::Key(Action::Sneak,true)]);
        r.end(1); r.end(2); r.clear();
        assert_eq!(r.drain(), [Event::Key(Action::Sneak,false)]);
    }
    #[test]
    fn toggle_single_and_focus_loss_emit_balanced_edges() {
        let mut l = Layout::default();
        l.buttons.iter_mut().find(|b| b.id==Id::Sneak).unwrap().behavior=Behavior::Toggle;
        let mut r=Router::default();
        r.begin(&l,1,center(&l,Id::Sneak),16.0/9.0,0); r.end(1);
        assert!(r.active(&Action::Sneak));
        r.begin(&l,2,center(&l,Id::Inventory),16.0/9.0,10); r.end(2);
        r.tick(109); assert!(r.active(&Action::Inventory));
        r.tick(110); assert!(!r.active(&Action::Inventory));
        r.clear(); assert!(!r.active(&Action::Sneak));
        assert!(r.drain().contains(&Event::Key(Action::Sneak,false)));
    }
}
