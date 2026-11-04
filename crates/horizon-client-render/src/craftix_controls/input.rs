//! Per-pointer ownership and ordered engine edges for the new controls.
use super::layout::{Action, Behavior, Button, Id, Layout};
use std::collections::{BTreeMap, BTreeSet};

/// Smallest camera drag, in height-relative units, that counts as a look.
const DRAG_THRESHOLD: f32 = 0.018;
/// A timed single press is released once the engine has sampled it.
const SINGLE_PRESS_MS: u64 = 100;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// An engine binding went down (`true`) or up (`false`).
    Key(Action, bool),
    /// Camera delta in fractions of the surface.
    Look([f32; 2]),
    /// Movement vector, right and up positive.
    Move([f32; 2]),
    /// An empty-surface gesture began; the host may turn it into a tap.
    LookBegin(u64, [f32; 2]),
    /// An empty-surface gesture ended; `dragged` is true once it turned the camera.
    LookEnd { pointer: u64, start: [f32; 2], dragged: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Owner {
    Finger(u64),
    Toggle(Id),
    Single(u64),
    Mapped(u64),
}

#[derive(Clone, Debug)]
enum Role {
    Joystick { center: [f32; 2], radius: f32 },
    Look { start: [f32; 2], last: [f32; 2], dragged: bool },
    Button,
    Toggle(Id),
    Ignore,
}

/// The replacement router. It is deliberately free of clocks and window
/// handles: the host supplies monotonic milliseconds and the surface aspect.
#[derive(Default)]
pub struct Router {
    pointers: BTreeMap<u64, Role>,
    owners: BTreeMap<Action, BTreeSet<Owner>>,
    toggles: BTreeMap<Id, Action>,
    singles: BTreeMap<u64, (Action, u64)>,
    next_single: u64,
    joystick: Option<u64>,
    movement: [f32; 2],
    /// How far the joystick was pushed, before the dead zone and clamping.
    reach: f32,
    inventory: bool,
    events: Vec<Event>,
}

impl Router {
    fn acquire(&mut self, action: Action, owner: Owner) {
        let owners = self.owners.entry(action.clone()).or_default();
        let first = owners.is_empty();
        owners.insert(owner);
        if first {
            self.events.push(Event::Key(action, true));
        }
    }
    fn release(&mut self, action: &Action, owner: Owner) {
        if let Some(owners) = self.owners.get_mut(action) {
            if !owners.remove(&owner) {
                return;
            }
            if owners.is_empty() {
                self.owners.remove(action);
                self.events.push(Event::Key(action.clone(), false));
            }
        }
    }
    /// Inventory overlays keep key buttons but hide gameplay-only ones.
    pub fn set_inventory(&mut self, inventory: bool) {
        if self.inventory != inventory {
            self.inventory = inventory;
            self.clear();
        }
    }
    pub fn inventory(&self) -> bool {
        self.inventory
    }
    pub fn owns(&self, id: u64) -> bool {
        self.pointers.contains_key(&id)
    }
    /// Start routing one pointer. `now_ms` is monotonic milliseconds.
    pub fn begin(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32, now_ms: u64) {
        if self.pointers.contains_key(&id)
            || !p.into_iter().all(f32::is_finite)
            || !aspect.is_finite()
            || aspect <= 0.0
        {
            return;
        }
        let role = match layout.hit_in(p, aspect, self.inventory) {
            Some(button) if button.id == Id::Joystick => {
                if self.joystick.is_some() {
                    Role::Ignore
                } else {
                    self.joystick = Some(id);
                    Role::Joystick { center: button.circle.center, radius: button.circle.radius }
                }
            }
            Some(button) => {
                if button.behavior == Behavior::Toggle
                    && self
                        .pointers
                        .values()
                        .any(|role| matches!(role, Role::Toggle(id) if *id == button.id))
                {
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
                            self.singles
                                .insert(single, (action.clone(), now_ms.saturating_add(SINGLE_PRESS_MS)));
                        }
                    }
                }
                if button.behavior == Behavior::Toggle {
                    Role::Toggle(button.id)
                } else {
                    Role::Button
                }
            }
            None => {
                self.events.push(Event::LookBegin(id, p));
                Role::Look { start: p, last: p, dragged: false }
            }
        };
        self.pointers.insert(id, role);
        if self.joystick == Some(id) {
            self.moved(layout, id, p, aspect);
        }
    }
    /// Hold an engine binding that was resolved by the host (inventory mode).
    pub fn begin_mapped(&mut self, id: u64, action: Action, p: [f32; 2]) -> bool {
        if self.pointers.contains_key(&id) || !p.into_iter().all(f32::is_finite) {
            return false;
        }
        if action.token().is_none() {
            return false;
        }
        self.pointers.insert(id, Role::Button);
        self.acquire(action, Owner::Mapped(id));
        true
    }
    pub fn moved(&mut self, layout: &Layout, id: u64, p: [f32; 2], aspect: f32) {
        if !p.into_iter().all(f32::is_finite) || !aspect.is_finite() || aspect <= 0.0 {
            return;
        }
        match self.pointers.get_mut(&id) {
            Some(Role::Joystick { center, radius }) => {
                let v = [
                    (p[0] - center[0]) * aspect / *radius,
                    (center[1] - p[1]) / *radius,
                ];
                let len = v[0].hypot(v[1]);
                let dead = layout.joystick_dead_zone.clamp(0.0, 0.5);
                let amount =
                    ((len - dead) / (1.0 - dead) * layout.joystick_sensitivity).clamp(0.0, 1.0);
                self.movement = if len > 0.0 {
                    [v[0] / len * amount, v[1] / len * amount]
                } else {
                    [0.0; 2]
                };
                self.reach = len;
                self.events.push(Event::Move(self.movement));
            }
            Some(Role::Look { start, last, dragged }) => {
                let travelled = ((p[0] - start[0]) * aspect).hypot(p[1] - start[1]);
                if !*dragged && travelled > DRAG_THRESHOLD {
                    *dragged = true;
                }
                if *dragged {
                    let delta = [
                        (p[0] - last[0]) * aspect * layout.camera_sensitivity,
                        (p[1] - last[1]) * layout.camera_sensitivity,
                    ];
                    self.events.push(Event::Look(delta));
                }
                *last = p;
            }
            _ => {}
        }
    }
    pub fn end(&mut self, id: u64) {
        match self.pointers.remove(&id) {
            Some(Role::Joystick { .. }) => {
                self.joystick = None;
                self.movement = [0.0; 2];
                self.reach = 0.0;
                self.events.push(Event::Move(self.movement));
            }
            Some(Role::Look { start, dragged, .. }) => {
                self.events.push(Event::LookEnd { pointer: id, start, dragged });
            }
            _ => {}
        }
        let held: Vec<_> = self
            .owners
            .iter()
            .filter_map(|(key, owners)| {
                (owners.contains(&Owner::Finger(id)) || owners.contains(&Owner::Mapped(id)))
                    .then(|| key.clone())
            })
            .collect();
        for key in held {
            self.release(&key, Owner::Finger(id));
            self.release(&key, Owner::Mapped(id));
        }
    }
    pub fn tick(&mut self, now_ms: u64) {
        let ready: Vec<_> = self
            .singles
            .iter()
            .filter(|(_, (_, deadline))| now_ms >= *deadline)
            .map(|(id, _)| *id)
            .collect();
        for id in ready {
            if let Some((action, _)) = self.singles.remove(&id) {
                self.release(&action, Owner::Single(id));
            }
        }
    }
    /// Call for focus loss, a context change, viewport change or shutdown.
    pub fn clear(&mut self) {
        for action in std::mem::take(&mut self.owners).into_keys() {
            self.events.push(Event::Key(action, false));
        }
        self.pointers.clear();
        self.toggles.clear();
        self.singles.clear();
        self.joystick = None;
        self.reach = 0.0;
        if self.movement != [0.0; 2] {
            self.movement = [0.0; 2];
            self.events.push(Event::Move(self.movement));
        }
    }
    pub fn movement(&self) -> [f32; 2] {
        self.movement
    }
    /// Unclamped joystick distance in radii; beyond 1.0 starts auto-sprint.
    pub fn reach(&self) -> f32 {
        self.reach
    }
    /// Reserve a pointer for a host hotspot so it never becomes a look gesture.
    pub fn ignore(&mut self, id: u64) {
        self.pointers.entry(id).or_insert(Role::Ignore);
    }
    pub fn active(&self, action: &Action) -> bool {
        self.owners.contains_key(action)
    }
    /// The buttons the player is currently holding, for the pressed visuals.
    pub fn held_button(&self, button: &Button) -> bool {
        button.action.as_ref().is_some_and(|action| self.active(action))
            || (button.id == Id::Joystick && self.joystick.is_some())
    }
    pub fn look_pointer(&self) -> Option<[f32; 2]> {
        self.pointers.values().find_map(|role| match role {
            Role::Look { last, .. } => Some(*last),
            _ => None,
        })
    }
    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::craftix_controls::layout::{Circle, Icon};

    fn center(l: &Layout, id: Id) -> [f32; 2] {
        l.button(id).unwrap().circle.center
    }
    fn custom_key(l: &mut Layout, token: &str, x: f32) -> Id {
        let id = Id::Custom(x as u32);
        let mut button = Button::new(id, Some(Action::Key(token.into())), [x, 0.30], 0.04);
        button.icon = Icon::Map;
        button.label = "Key".into();
        l.buttons.push(button);
        id
    }

    #[test]
    fn joystick_jump_attack_and_camera_own_separate_pointers() {
        let l = Layout::default();
        let mut r = Router::default();
        let aspect = 16.0 / 9.0;
        let joy = center(&l, Id::Joystick);
        r.begin(&l, 1, joy, aspect, 0);
        r.moved(&l, 1, [joy[0], joy[1] - 0.1], aspect);
        r.begin(&l, 2, center(&l, Id::Jump), aspect, 0);
        r.begin(&l, 3, center(&l, Id::Attack), aspect, 0);
        r.begin(&l, 4, [0.5, 0.5], aspect, 0);
        r.moved(&l, 4, [0.55, 0.52], aspect);
        let e = r.drain();
        assert!(e.contains(&Event::Key(Action::Jump, true)));
        assert!(e.contains(&Event::Key(Action::Attack, true)));
        assert!(e.iter().any(|e| matches!(e, Event::Look(_))));
        assert!(e.contains(&Event::LookBegin(4, [0.5, 0.5])));
        assert!(r.movement()[1] > 0.0);
        r.end(2);
        assert!(r.active(&Action::Attack));
        r.clear();
        assert_eq!(r.movement(), [0.0, 0.0]);
        assert!(r.drain().contains(&Event::Key(Action::Attack, false)));
    }

    #[test]
    fn a_short_touch_without_drag_is_a_tap_not_a_camera_turn() {
        let l = Layout::default();
        let mut r = Router::default();
        let aspect = 16.0 / 9.0;
        r.begin(&l, 1, [0.5, 0.5], aspect, 0);
        r.moved(&l, 1, [0.501, 0.501], aspect);
        r.end(1);
        let e = r.drain();
        assert!(!e.iter().any(|e| matches!(e, Event::Look(_))));
        assert!(e.contains(&Event::LookEnd { pointer: 1, start: [0.5, 0.5], dragged: false }));
    }

    #[test]
    fn camera_drag_is_dragged_and_reports_the_delta() {
        let l = Layout::default();
        let mut r = Router::default();
        let aspect = 16.0 / 9.0;
        r.begin(&l, 1, [0.5, 0.5], aspect, 0);
        r.moved(&l, 1, [0.6, 0.5], aspect);
        r.end(1);
        let e = r.drain();
        match e.iter().find(|e| matches!(e, Event::Look(_))) {
            Some(Event::Look(delta)) => assert!((delta[0] - 0.1 * aspect).abs() < 1e-5),
            _ => panic!("no camera delta"),
        }
        assert!(e.contains(&Event::LookEnd { pointer: 1, start: [0.5, 0.5], dragged: true }));
    }

    #[test]
    fn two_different_buttons_share_one_action_without_premature_release() {
        let l = {
            let mut l = Layout::default();
            custom_key(&mut l, "key.keyboard.m", 0.55);
            custom_key(&mut l, "key.keyboard.m", 0.66);
            l
        };
        let mut r = Router::default();
        r.begin(&l, 1, [0.55, 0.30], 16.0 / 9.0, 0);
        r.begin(&l, 2, [0.66, 0.30], 16.0 / 9.0, 0);
        assert_eq!(
            r.drain(),
            [Event::Key(Action::Key("key.keyboard.m".into()), true)]
        );
        r.end(1);
        assert!(r.drain().is_empty());
        r.end(2);
        assert_eq!(
            r.drain(),
            [Event::Key(Action::Key("key.keyboard.m".into()), false)]
        );
    }

    #[test]
    fn reused_pointer_id_does_not_orphan_timed_single_press() {
        let l = Layout::default();
        let mut r = Router::default();
        r.begin(&l, 1, center(&l, Id::Inventory), 16.0 / 9.0, 0);
        r.end(1);
        r.begin(&l, 1, center(&l, Id::More), 16.0 / 9.0, 1);
        r.end(1);
        r.tick(101);
        assert!(!r.active(&Action::Inventory) && !r.active(&Action::More));
    }

    #[test]
    fn two_fingers_on_one_toggle_do_not_flip_it_twice() {
        let mut l = Layout::default();
        l.button_mut(Id::Sneak).unwrap().behavior = Behavior::Toggle;
        let mut r = Router::default();
        let p = center(&l, Id::Sneak);
        r.begin(&l, 1, p, 16.0 / 9.0, 0);
        r.begin(&l, 2, p, 16.0 / 9.0, 0);
        assert_eq!(r.drain(), [Event::Key(Action::Sneak, true)]);
        r.end(1);
        r.end(2);
        r.clear();
        assert_eq!(r.drain(), [Event::Key(Action::Sneak, false)]);
    }

    #[test]
    fn toggle_single_and_focus_loss_emit_balanced_edges() {
        let mut l = Layout::default();
        l.button_mut(Id::Sneak).unwrap().behavior = Behavior::Toggle;
        let mut r = Router::default();
        r.begin(&l, 1, center(&l, Id::Sneak), 16.0 / 9.0, 0);
        r.end(1);
        assert!(r.active(&Action::Sneak));
        r.begin(&l, 2, center(&l, Id::Inventory), 16.0 / 9.0, 10);
        r.end(2);
        r.tick(109);
        assert!(r.active(&Action::Inventory));
        r.tick(110);
        assert!(!r.active(&Action::Inventory));
        r.clear();
        assert!(!r.active(&Action::Sneak));
        assert!(r.drain().contains(&Event::Key(Action::Sneak, false)));
    }

    #[test]
    fn one_joystick_at_a_time_and_mapped_inventory_holds() {
        let l = Layout::default();
        let mut r = Router::default();
        let aspect = 16.0 / 9.0;
        let joy = center(&l, Id::Joystick);
        r.begin(&l, 1, joy, aspect, 0);
        r.begin(&l, 2, joy, aspect, 0);
        r.moved(&l, 2, [joy[0], joy[1] - 0.2], aspect);
        assert_eq!(r.movement(), [0.0, 0.0]);
        r.moved(&l, 1, [joy[0], joy[1] - 0.2], aspect);
        assert!(r.movement()[1] > 0.0);
        assert!(r.begin_mapped(3, Action::Key("action:key.attack".into()), [0.5, 0.5]));
        assert!(!r.begin_mapped(4, Action::Jump, [0.5, 0.5]));
        r.end(3);
        let e = r.drain();
        assert!(e.contains(&Event::Key(Action::Key("action:key.attack".into()), true)));
        assert!(e.contains(&Event::Key(Action::Key("action:key.attack".into()), false)));
    }

    #[test]
    fn inventory_hiding_clears_held_gameplay_buttons() {
        let mut l = Layout::default();
        l.button_mut(Id::Inventory).unwrap().visibility = super::super::layout::Visibility::Gameplay;
        let mut r = Router::default();
        r.begin(&l, 1, center(&l, Id::Inventory), 16.0 / 9.0, 0);
        assert!(r.active(&Action::Inventory));
        r.set_inventory(true);
        assert!(!r.active(&Action::Inventory));
        assert!(r.drain().contains(&Event::Key(Action::Inventory, false)));
        assert!(l.hit_in(center(&l, Id::Inventory), 16.0 / 9.0, true).is_none());
    }

    #[test]
    fn circle_geometry_is_used_for_hits() {
        let l = Layout::default();
        let attack = l.button(Id::Attack).unwrap().circle;
        let inside = Circle { center: attack.center, radius: attack.radius };
        assert!(inside.contains(attack.center, 16.0 / 9.0));
        let outside = [attack.center[0] + attack.radius, attack.center[1] + attack.radius];
        assert!(!inside.contains(outside, 16.0 / 9.0));
    }
}
