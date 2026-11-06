//! Per-pointer ownership and ordered engine edges for the new controls.
//!
//! Touch semantics follow Zalith Launcher 2's `TouchProcessor` /
//! `PointerEventBus` / `InteractionBehavior` contract: topmost widget hit
//! testing, penetrable widgets that let the camera keep the pointer,
//! out-of-bounds release for swipable widgets with re-press on return,
//! swipe chains between swipable widgets, and toggle widgets that block
//! chains. The engine edges stay CRAFTIX's own vocabulary.
use super::layout::{Action, Behavior, Id, Layout};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Event { Key(Action, bool), Look([f32; 2]), Move([f32; 2]) }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Owner { Finger(u64), Toggle(Id), Single(u64), Lock }
#[derive(Clone, Debug)]
enum Role {
    Joystick { center: [f32; 2], radius: f32 },
    Look([f32; 2]),
    /// Active widget chain for one finger; `look` is Some when the pressed
    /// widget is penetrable and the camera keeps receiving the pointer.
    Button { look: Option<[f32; 2]>, chain: Vec<Id> },
    Toggle(Id),
    Ignore,
}
#[derive(Default)]
pub struct Router {
    pointers: BTreeMap<u64, Role>,
    owners: BTreeMap<Action, BTreeSet<Owner>>,
    toggles: BTreeMap<Id, Action>,
    singles: BTreeMap<u64, (Action, u64)>,
    next_single: u64,
    joystick: Option<u64>,
    /// Auto-run lock (Zalith joystick lockEvents): forward movement plus
    /// the sprint action until the joystick is touched again.
    locked: bool,
    /// A strong north flick happened while the stick was held; the lock
    /// arms on lift.
    lock_pending: bool,
    movement: [f32; 2],
    events: Vec<Event>,
    /// Last host-supplied monotonic time; Single re-presses during moved()
    /// schedule against it because moved() carries no timestamp.
    last_now: u64,
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
    fn unlock(&mut self) {
        if self.locked {
            self.locked = false;
            self.release(&Action::Sprint, Owner::Lock);
            self.movement = [0.0; 2];
            self.events.push(Event::Move(self.movement));
        }
    }
    /// Monotonic milliseconds, supplied by the host so tests do not sleep.
    pub fn begin(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32, now_ms: u64) {
        self.begin_for(layout, id, p, aspect, now_ms, false);
    }
    /// In menus, empty space belongs to the menu, not the camera.
    pub fn begin_for(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32, now_ms: u64, inventory: bool) {
        if self.pointers.contains_key(&id) || !p.into_iter().all(|v| v.is_finite() && (0.0..=1.0).contains(&v)) || !aspect.is_finite() || aspect <= 0.0 { return; }
        self.last_now = now_ms;
        let role = match layout.hit_for(p, aspect, inventory) {
            Some(button) if button.id == Id::Joystick && inventory => return,
            Some(button) if button.id == Id::Joystick => {
                if self.joystick.is_some() { Role::Ignore }
                else {
                    self.unlock();
                    self.joystick = Some(id);
                    Role::Joystick { center: button.center(aspect), radius: button.half_height() }
                }
            }
            Some(button) => {
                if button.behavior == Behavior::Toggle && self.pointers.values().any(|role| matches!(role, Role::Toggle(id) if *id == button.id)) {
                    self.pointers.insert(id, Role::Ignore);
                    return;
                }
                let mut chain = Vec::new();
                if let Some(action) = &button.action {
                    match button.behavior {
                        Behavior::Hold | Behavior::Swipe => {
                            self.acquire(action.clone(), Owner::Finger(id));
                            chain.push(button.id);
                        }
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
                if button.behavior == Behavior::Toggle { Role::Toggle(button.id) }
                else {
                    Role::Button { look: button.penetrable.then_some(p), chain }
                }
            }
            None if inventory => return,
            None => Role::Look(p),
        };
        self.pointers.insert(id, role);
        if self.joystick == Some(id) { self.moved(layout, id, p, aspect); }
    }
    pub fn moved(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32) {
        if !p.into_iter().all(f32::is_finite) || !aspect.is_finite() || aspect <= 0.0 { return; }
        // The pointer can leave the window while held; bound extreme finite
        // coordinates too, to prevent overflow and NaN movement/camera deltas.
        let p = p.map(|v| v.clamp(0.0, 1.0));
        // Button chains release/acquire through &mut self, so their state is
        // cloned out of the map before any whole-struct borrow.
        let button_state = match self.pointers.get(&id) {
            Some(Role::Button { look, chain }) => Some((*look, chain.clone())),
            _ => None,
        };
        if let Some((mut look, mut chain)) = button_state {
            // Camera keeps receiving the pointer while a penetrable widget
            // is held (Zalith isPenetrable pass-through).
            if let Some(last) = look.as_mut() {
                let delta = [(p[0] - last[0]) * aspect * layout.camera_sensitivity, (p[1] - last[1]) * layout.camera_sensitivity];
                *last = p;
                self.events.push(Event::Look(delta));
            }
            // Out-of-bounds release for swipable widgets (TouchProcessor).
            let mut released = Vec::new();
            for button_id in chain.iter() {
                if let Some(button) = layout.buttons.iter().find(|b| b.id == *button_id) {
                    if button.behavior.release_on_out_of_bounds() && !button.contains(p, aspect) {
                        released.push(*button_id);
                    }
                }
            }
            for button_id in released {
                chain.retain(|c| *c != button_id);
                if let Some(button) = layout.buttons.iter().find(|b| b.id == button_id) {
                    if let Some(action) = &button.action {
                        self.release(action, Owner::Finger(id));
                    }
                }
            }
            if chain.is_empty() {
                // No widget holds this finger any more: the event is
                // unconsumed and the camera takes over; touching a widget
                // again starts a fresh press (onPointerBackInBounds).
                match layout.hit_for(p, aspect, false) {
                    Some(button) if button.id != Id::Joystick => {
                        let mut fresh = Vec::new();
                        let mut penetrable = button.penetrable;
                        if let Some(action) = &button.action {
                            match button.behavior {
                                Behavior::Hold | Behavior::Swipe => {
                                    self.acquire(action.clone(), Owner::Finger(id));
                                    fresh.push(button.id);
                                }
                                Behavior::Toggle => {
                                    if !self.toggles.contains_key(&button.id) {
                                        self.toggles.insert(button.id, action.clone());
                                        self.acquire(action.clone(), Owner::Toggle(button.id));
                                    } else { penetrable = false; }
                                }
                                Behavior::Single => {
                                    let single = self.next_single;
                                    self.next_single = self.next_single.wrapping_add(1);
                                    self.acquire(action.clone(), Owner::Single(single));
                                    self.singles.insert(single, (action.clone(), self.last_now.saturating_add(100)));
                                }
                            }
                        }
                        self.pointers.insert(id, Role::Button {
                            look: penetrable.then_some(p), chain: fresh,
                        });
                        return;
                    }
                    _ => {
                        self.pointers.insert(id, Role::Look(p));
                        return;
                    }
                }
            }
            // Swipe-in: the finger slides onto another swipable widget
            // while no active widget blocks the chain (Toggle blocks).
            let blocked = chain.iter().any(|cid|
                layout.buttons.iter().find(|b| b.id == *cid).is_some_and(|b| b.behavior.blocks_swipe_chain()));
            if !blocked
                && let Some(button) = layout.hit_for(p, aspect, false)
                && button.behavior.can_be_swiped_to()
                && button.id != Id::Joystick
                && !chain.contains(&button.id)
            {
                if let Some(action) = &button.action {
                    self.acquire(action.clone(), Owner::Finger(id));
                }
                chain.push(button.id);
            }
            self.pointers.insert(id, Role::Button { look, chain });
            return;
        }
        // Joystick and camera motion: field-disjoint borrows only.
        match self.pointers.get_mut(&id) {
            Some(Role::Joystick { center, radius }) => {
                let center = *center;
                let radius = (*radius).max(1e-6);
                let v = [(p[0] - center[0]) * aspect / radius, (center[1] - p[1]) / radius];
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
        // Joystick forward lock (Zalith lockEvents): a strong north flick
        // keeps walking forward with sprint after the finger lifts.
        if matches!(self.pointers.get(&id), Some(Role::Joystick { .. })) {
            self.lock_pending =
                self.movement[1] >= 0.5 && self.movement[0].abs() <= self.movement[1] * 0.5;
        }
    }
    pub fn end(&mut self, id: u64) {
        if matches!(self.pointers.remove(&id), Some(Role::Joystick { .. })) {
            self.joystick = None;
            if self.lock_pending {
                self.lock_pending = false;
                self.locked = true;
                self.movement = [0.0, 1.0];
                self.events.push(Event::Move(self.movement));
                self.acquire(Action::Sprint, Owner::Lock);
            } else {
                self.unlock();
            }
        }
        let held: Vec<_> = self.owners.iter().filter(|(_, owners)| owners.contains(&Owner::Finger(id)))
            .map(|(key, _)| key.clone()).collect();
        for key in held { self.release(&key, Owner::Finger(id)); }
        self.lock_pending = self.lock_pending && self.locked == false && self.joystick.is_some();
    }
    pub fn tick(&mut self, now_ms: u64) {
        self.last_now = now_ms;
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
        self.locked = false; self.lock_pending = false;
        if self.movement != [0.0; 2] { self.movement = [0.0; 2]; self.events.push(Event::Move(self.movement)); }
    }
    pub fn owns_pointer(&self, id: u64) -> bool { self.pointers.contains_key(&id) }
    pub fn movement(&self) -> [f32; 2] { self.movement }
    pub fn joystick_locked(&self) -> bool { self.locked }
    pub fn active(&self, action: &Action) -> bool { self.owners.contains_key(action) }
    pub fn drain(&mut self) -> Vec<Event> { std::mem::take(&mut self.events) }
}
fn inventory_hit(_layout: &Layout, _p: [f32; 2], _aspect: f32) -> bool { false }
#[cfg(test)]
mod tests {
    use super::*;
    use crate::craftix_controls::layout::{Button, Circle, Rect};
    fn center(l: &Layout, id: Id) -> [f32; 2] {
        l.buttons.iter().find(|b| b.id == id).map(|b| b.center(16.0 / 9.0)).unwrap()
    }
    fn push(l: &mut Layout, button: Button) { l.buttons.push(button); }
    fn custom_key(id: u32, anchor: [f32; 2], size: [f32; 2], behavior: Behavior, key: &str) -> Button {
        Button { id: Id::Custom(id), action: Some(Action::Key(key.into())),
            rect: Some(Rect { anchor, size }),
            circle: Circle { center: Rect { anchor, size }.center(16.0 / 9.0), radius: size[1] * 0.5 },
            opacity: 1.0, behavior, icon: super::super::layout::Icon::Map, icon_rgba_16: None,
            enabled: true, visibility: super::super::layout::Visibility::Always,
            label: key.into(), penetrable: false, group: 0 }
    }
    fn custom(id: u32, anchor: [f32; 2], size: [f32; 2], behavior: Behavior) -> Button {
        custom_key(id, anchor, size, behavior, "key.keyboard.m")
    }
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
        push(&mut l, custom(10, [0.60, 0.25], [0.08, 0.08], Behavior::Hold));
        push(&mut l, custom(11, [0.75, 0.25], [0.08, 0.08], Behavior::Hold));
        let mut r = Router::default();
        r.begin(&l,1,center(&l,Id::Custom(10)),16.0/9.0,0);
        r.begin(&l,2,center(&l,Id::Custom(11)),16.0/9.0,0);
        assert_eq!(r.drain(), [Event::Key(Action::Key("key.keyboard.m".into()),true)]);
        r.end(1); assert!(r.drain().is_empty()); r.end(2);
        assert_eq!(r.drain(), [Event::Key(Action::Key("key.keyboard.m".into()),false)]);
    }
    #[test]
    fn inventory_visibility_and_extreme_coordinates_are_safe() {
        let mut l=Layout::default();
        let jump=l.buttons.iter_mut().find(|b| b.id==Id::Jump).unwrap();
        jump.visibility=super::super::layout::Visibility::Inventory;
        let p=jump.center(16.0/9.0); let mut r=Router::default();
        r.begin_for(&l,1,p,16.0/9.0,0,true);
        assert!(r.active(&Action::Jump)); r.end(1); r.drain();
        r.begin_for(&l,2,[0.5,0.5],16.0/9.0,0,true);
        r.moved(&l,2,[1.0,1.0],16.0/9.0); assert!(r.drain().is_empty());
        let joy=center(&l,Id::Joystick); r.begin(&l,3,joy,16.0/9.0,0);
        r.moved(&l,3,[f32::MAX,-f32::MAX],16.0/9.0);
        assert!(r.movement().into_iter().all(f32::is_finite));
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
        let mut l=Layout::default();
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
    #[test]
    fn swipable_widget_releases_out_of_bounds_and_represses_on_return() {
        let mut l = Layout::default();
        push(&mut l, custom(20, [0.2, 0.5], [0.15, 0.15], Behavior::Swipe));
        let mut r = Router::default();
        let p = center(&l, Id::Custom(20));
        r.begin(&l, 1, p, 16.0/9.0, 0);
        assert!(r.active(&Action::Key("key.keyboard.m".into())));
        // Finger leaves the widget: the swipable button releases...
        r.moved(&l, 1, [0.9, 0.1], 16.0/9.0);
        assert!(!r.active(&Action::Key("key.keyboard.m".into())));
        let e = r.drain();
        assert_eq!(e.last(), Some(&Event::Key(Action::Key("key.keyboard.m".into()), false)));
        // ...and the camera takes over until the finger returns.
        assert!(matches!(e.iter().find(|ev| matches!(ev, Event::Look(_))), Some(_)));
        r.moved(&l, 1, p, 16.0/9.0);
        assert!(r.active(&Action::Key("key.keyboard.m".into())));
        r.end(1);
        assert!(!r.active(&Action::Key("key.keyboard.m".into())));
    }
    #[test]
    fn swipe_chains_between_swipable_widgets_and_hold_is_freshly_pressed() {
        let mut l = Layout::default();
        push(&mut l, custom_key(30, [0.30, 0.5], [0.12, 0.12], Behavior::Swipe, "key.keyboard.a"));
        push(&mut l, custom_key(31, [0.48, 0.5], [0.12, 0.12], Behavior::Swipe, "key.keyboard.b"));
        let mut r = Router::default();
        r.begin(&l, 1, center(&l, Id::Custom(30)), 16.0/9.0, 0);
        assert!(r.active(&Action::Key("key.keyboard.a".into())));
        // Slide onto the second swipable widget: the chain keeps A held
        // and acquires B (PointerEventBus swipe chain).
        r.moved(&l, 1, center(&l, Id::Custom(31)), 16.0/9.0);
        assert!(r.active(&Action::Key("key.keyboard.a".into())));
        assert!(r.active(&Action::Key("key.keyboard.b".into())));
        r.end(1);
        assert!(!r.active(&Action::Key("key.keyboard.a".into())));
        assert!(!r.active(&Action::Key("key.keyboard.b".into())));
        // Sliding off a swipable widget onto a Hold widget releases the
        // first out of bounds and starts a fresh press on the Hold one;
        // Hold never joins a chain via swipe-in.
        let mut l2 = Layout::default();
        push(&mut l2, custom_key(32, [0.30, 0.5], [0.12, 0.12], Behavior::Swipe, "key.keyboard.a"));
        push(&mut l2, custom_key(33, [0.48, 0.5], [0.12, 0.12], Behavior::Hold, "key.keyboard.b"));
        push(&mut l2, custom_key(34, [0.66, 0.5], [0.12, 0.12], Behavior::Swipe, "key.keyboard.c"));
        let mut r2 = Router::default();
        r2.begin(&l2, 1, center(&l2, Id::Custom(32)), 16.0/9.0, 0);
        r2.moved(&l2, 1, center(&l2, Id::Custom(33)), 16.0/9.0);
        assert!(!r2.active(&Action::Key("key.keyboard.a".into())));
        assert!(r2.active(&Action::Key("key.keyboard.b".into())));
        r2.end(1);
        assert!(!r2.active(&Action::Key("key.keyboard.b".into())));
    }
    #[test]
    fn penetrable_widget_lets_the_camera_move_while_pressed() {
        let mut l = Layout::default();
        let mut b = custom(40, [0.7, 0.4], [0.16, 0.16], Behavior::Hold);
        b.penetrable = true;
        push(&mut l, b);
        let mut r = Router::default();
        let p = center(&l, Id::Custom(40));
        r.begin(&l, 1, p, 16.0/9.0, 0);
        r.moved(&l, 1, [p[0] + 0.05, p[1] + 0.03], 16.0/9.0);
        let e = r.drain();
        assert!(e.contains(&Event::Key(Action::Key("key.keyboard.m".into()), true)));
        assert!(e.iter().any(|ev| matches!(ev, Event::Look(_))));
        r.end(1);
        assert!(!r.active(&Action::Key("key.keyboard.m".into())));
    }
    #[test]
    fn north_flick_locks_forward_run_until_the_stick_is_touched_again() {
        let l = Layout::default(); let mut r = Router::default(); let aspect = 16.0/9.0;
        let joy = center(&l, Id::Joystick);
        r.begin(&l, 1, joy, aspect, 0);
        r.moved(&l, 1, [joy[0], joy[1] - 0.3], aspect);
        r.end(1);
        assert!(r.joystick_locked());
        assert_eq!(r.movement(), [0.0, 1.0]);
        assert!(r.active(&Action::Sprint));
        // Touching the stick again unlocks and returns control to the finger.
        r.begin(&l, 2, joy, aspect, 10);
        assert!(!r.joystick_locked());
        assert!(!r.active(&Action::Sprint));
        r.end(2);
        assert_eq!(r.movement(), [0.0, 0.0]);
    }
    #[test]
    fn hidden_groups_are_never_hit() {
        let mut l = Layout::default();
        let p = center(&l, Id::Chat);
        assert!(l.hit_for(p, 16.0/9.0, false).is_some());
        l.hidden_groups.insert(1);
        assert!(l.hit_for(p, 16.0/9.0, false).is_none());
    }
}
