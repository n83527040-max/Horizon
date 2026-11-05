//! Opt-in integration of the replacement touch core. This is not the default
//! until the new editor, migration and runtime/device tests are complete.
use super::Game;
use horizon_client::craftix_controls::{
    bridge::{Bridge, HostEvent, UiAction}, editor::Layouts,
    input::Router, paint::Animator,
};
use std::{collections::BTreeSet, sync::OnceLock, time::Instant};
use winit::event::{Touch, TouchPhase};

pub(super) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("CRAFTIX_EXPERIMENTAL_TOUCH").is_ok_and(|v| v == "1"))
}

pub(super) struct Controls {
    pub(super) layouts: Layouts,
    pub(super) router: Router,
    pub(super) bridge: Bridge,
    pub(super) animation: Animator,
    pub(super) last_frame: Instant,
    pub(super) reserved: BTreeSet<u64>,
    pub(super) viewport: Option<[u32; 2]>,
}
impl Default for Controls {
    fn default() -> Self {
        // Separate file: never overwrite or implicitly replace legacy settings.
        let mut layouts = std::fs::read("craftix-controls-v1.json").ok()
            .and_then(|bytes| Layouts::from_json(&bytes).ok()).unwrap_or_default();
        // Do not advertise an action which the experimental path cannot yet do.
        for layout in &mut layouts.items {
            for button in &mut layout.buttons {
                if button.id == horizon_client::craftix_controls::layout::Id::More { button.enabled = false; }
            }
        }
        Self { layouts, router: Router::default(), bridge: Bridge::default(), animation: Animator::default(),
            last_frame: Instant::now(), reserved: BTreeSet::new(), viewport: None }
    }
}
impl Game<'_> {
    fn craftix_dispatch(&mut self, events: Vec<HostEvent>, width: u32, height: u32) {
        for event in events {
            match event {
                HostEvent::Input(name, down) => self.android_dispatch_touch_inputs([(name, down)]),
                HostEvent::Move(_) => {}, // the latest state is merged in poll_pads
                HostEvent::Look(delta) => {
                    let factor = height as f32 / width.max(height).max(1) as f32 * 180.0;
                    let dx = delta[0] * factor;
                    let dy = delta[1] * factor;
                    self.tutorial.mouse(f64::from(dx), f64::from(dy));
                    self.yaw += dx;
                    self.pitch = (self.pitch + dy).clamp(-90.0, 90.0);
                }
                HostEvent::Ui(UiAction::Pause, true) => {
                    if let Err(error) = self.pad_back() { log::warn!("Touch pause: {error}"); }
                }
                HostEvent::Ui(UiAction::Settings, true) => self.open_options(false),
                // Not wired to the legacy editor: it edits a different model.
                HostEvent::Ui(UiAction::More, true) => log::warn!("Replacement touch editor is not available yet"),
                HostEvent::Ui(_, false) => {},
            }
        }
    }
    pub(super) fn craftix_clear(&mut self) {
        self.pad.craftix.router.clear();
        let events = self.pad.craftix.router.drain();
        let mut events = self.pad.craftix.bridge.translate(events);
        events.extend(self.pad.craftix.bridge.clear());
        self.pad.craftix.reserved.clear();
        self.craftix_dispatch(events, 1, 1);
    }
    pub(super) fn craftix_tick(&mut self) {
        let now = self.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
        self.pad.craftix.router.tick(now);
        let events = self.pad.craftix.router.drain();
        let events = self.pad.craftix.bridge.translate(events);
        let size = self.window.as_ref().map(|w| w.inner_size());
        self.craftix_dispatch(events, size.map_or(1, |s| s.width), size.map_or(1, |s| s.height));
    }
    pub(super) fn craftix_touch(&mut self, touch: &Touch, size: [u32; 2]) {
        if self.pad.craftix.viewport.is_some_and(|old| old != size) { self.craftix_clear(); }
        self.pad.craftix.viewport = Some(size);
        let point = [touch.location.x as f32 / size[0].max(1) as f32,
            touch.location.y as f32 / size[1].max(1) as f32];
        let aspect = size[0].max(1) as f32 / size[1].max(1) as f32;
        if touch.phase == TouchPhase::Cancelled { self.craftix_clear(); return; }
        if self.pad.craftix.reserved.contains(&touch.id) {
            if touch.phase == TouchPhase::Ended { self.pad.craftix.reserved.remove(&touch.id); }
            return;
        }
        if touch.phase == TouchPhase::Started {
            let layout = &self.pad.craftix.layouts.items[self.pad.craftix.layouts.active];
            let batch = horizon_client::hud::HudBatch::new(size);
            let p = [touch.location.x as f32 / batch.scale, touch.location.y as f32 / batch.scale];
            if layout.hit(point, aspect).is_none()
                && let Some(slot) = horizon_client::touch::hotbar_slot(batch.width, batch.height, p)
            {
                self.selected = slot;
                self.pad.craftix.reserved.insert(touch.id);
                return;
            }
        }
        let layout = &self.pad.craftix.layouts.items[self.pad.craftix.layouts.active];
        match touch.phase {
            TouchPhase::Started => self.pad.craftix.router.begin(layout, touch.id, point, aspect,
                self.started.elapsed().as_millis().min(u64::MAX as u128) as u64),
            TouchPhase::Moved => self.pad.craftix.router.moved(layout, touch.id, point, aspect),
            TouchPhase::Ended => self.pad.craftix.router.end(touch.id),
            TouchPhase::Cancelled => unreachable!(),
        }
        let events = self.pad.craftix.router.drain();
        let events = self.pad.craftix.bridge.translate(events);
        self.pad.in_use = true;
        self.craftix_dispatch(events, size[0], size[1]);
    }
}
