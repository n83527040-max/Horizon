//! Integration of the replacement touch core. The replacement is the default
//! control path; `CRAFTIX_EXPERIMENTAL_TOUCH=0` (env or compile-time) opts
//! back out to the legacy touch system for rollback.
use super::{Game, Screen};
use horizon_client::craftix_controls::{
    bridge::{Bridge, HostEvent, UiAction}, editor::Layouts,
    input::Router, paint::Animator, ui::{EditorUi, UiEvent},
};
use std::{collections::BTreeSet, sync::OnceLock, time::Instant};
use winit::event::{Touch, TouchPhase};

pub(super) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let raw = std::env::var("CRAFTIX_EXPERIMENTAL_TOUCH")
            .ok()
            .or_else(|| option_env!("CRAFTIX_EXPERIMENTAL_TOUCH").map(str::to_owned));
        // Default on: only an explicit "0" rolls back to the legacy path.
        let on = !matches!(raw.as_deref(), Some("0"));
        log::info!(
            "CRAFTIX replacement controls: {}",
            if on { "active" } else { "disabled by CRAFTIX_EXPERIMENTAL_TOUCH=0" }
        );
        on
    })
}

/// Whether the replacement HUD shows during gameplay.
///
/// Driven by the configured input preference, not whichever device was used
/// last: in `Auto`/`Touch` the HUD stays visible while a physical mouse moves
/// or clicks, so touch buttons and mouse input work at the same time. An
/// explicit `KeyboardMouse`/`Controller` preference keeps the HUD hidden, the
/// same as before.
pub(super) fn hud_visible() -> bool {
    matches!(
        horizon_client::android_runtime::input_mode(),
        horizon_client::android_gamepad::InputMode::Auto
            | horizon_client::android_gamepad::InputMode::Touch
    )
}

pub(super) struct Controls {
    pub(super) layouts: Layouts,
    pub(super) editor: Option<EditorUi>,
    /// An invalid existing file must not be silently overwritten by defaults.
    load_error: Option<String>,
    pub(super) router: Router,
    pub(super) bridge: Bridge,
    pub(super) animation: Animator,
    pub(super) last_frame: Instant,
    pub(super) reserved: BTreeSet<u64>,
    pub(super) viewport: Option<[u32; 2]>,
    inventory_active: bool,
}
impl Default for Controls {
    fn default() -> Self {
        // Separate file: never overwrite or implicitly replace legacy settings.
        let (layouts, load_error) = match Layouts::load(std::path::Path::new("craftix-controls-v1.json")) {
            Ok(layouts) => (layouts, None),
            Err(error) if error.kind() == std::io::ErrorKind::InvalidData =>
                (Layouts::default(), Some(format!("Existing controls file invalid; it was not overwritten: {error}"))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // Preserve every old file; conversion only exists in memory until Save.
                let legacy = std::fs::read("horizon-touch.json").ok()
                    .and_then(|bytes| serde_json::from_slice::<horizon_client::touch::Settings>(&bytes).ok());
                let base=legacy.unwrap_or_default();
                let converted=horizon_client::craftix_controls::migrate::preview(&base);
                for warning in &converted.warnings { log::warn!("Touch migration: {warning}"); }
                let mut layouts=converted.layouts;
                let dir=std::path::Path::new("craftix-controls");
                if std::fs::symlink_metadata(dir).is_ok_and(|m| !m.file_type().is_symlink() && m.is_dir()) {
                    for slot in 1..=3 {
                        let path=dir.join(format!("slot-{slot}.json"));
                        if std::fs::symlink_metadata(&path).is_ok_and(|m| !m.file_type().is_symlink() && m.is_file() && m.len() <= 128*1024) {
                            match std::fs::read(path).and_then(|bytes| horizon_client::craftix_controls::migrate::add_slot(&mut layouts,&base,&bytes,slot)) {
                                Ok(warnings) => for warning in warnings { log::warn!("Touch slot {slot}: {warning}"); },
                                Err(error) => log::warn!("Touch slot {slot} was left unchanged: {error}"),
                            }
                        }
                    }
                }
                (layouts,None)
            }
            Err(error) => (Layouts::default(), Some(format!("Cannot read existing controls file: {error}"))),
        };
        Self { layouts, editor: None, load_error, router: Router::default(), bridge: Bridge::default(), animation: Animator::default(),
            last_frame: Instant::now(), reserved: BTreeSet::new(), viewport: None, inventory_active: false }
    }
}
impl Game<'_> {
    fn craftix_dispatch(&mut self, events: Vec<HostEvent>, width: u32, height: u32) {
        for event in events {
            match event {
                HostEvent::Input(name, down) => self.android_dispatch_craftix_inputs([(name, down)]),
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
                HostEvent::Ui(UiAction::More, true) => {
                    self.craftix_clear();
                    match EditorUi::new(self.pad.craftix.layouts.clone()) {
                        Ok(mut editor) => {
                            if let Some(error) = &self.pad.craftix.load_error { editor.error = error.clone(); }
                            log::info!("CRAFTIX controls editor opened");
                            self.pad.craftix.editor = Some(editor);
                        },
                        Err(error) => log::warn!("Open touch editor: {error}"),
                    }
                },
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
    pub(super) fn craftix_inventory_allowed(&self) -> bool {
        matches!(self.screen, Screen::Inventory | Screen::Creative) && self.title.is_none()
            && self.sign_edit.is_none() && self.book.is_none() && !self.dialog_open()
            && self.server_packs.prompt.is_none() && self.link_prompt.is_none()
            && self.pad.craftix.editor.is_none() && !crate::platform::active()
    }
    pub(super) fn craftix_context(&mut self, inventory: bool) {
        if self.pad.craftix.inventory_active != inventory {
            self.craftix_clear();
            self.pad.craftix.inventory_active = inventory;
        }
    }
    pub(super) fn craftix_inventory_layout(&self, batch: &horizon_client::hud::HudBatch)
        -> horizon_client::craftix_controls::layout::Layout {
        let image = self.container_image(batch);
        // Reserve the container and its recipe/merchant panel. Controls that
        // intersect this conservative region are neither drawn nor hit-tested.
        let blocked = [image.left as f32 - 160.0, image.top as f32 - 24.0,
            (image.left + image.width) as f32 + 32.0, (image.top + image.height) as f32 + 24.0];
        self.pad.craftix.layouts.items[self.pad.craftix.layouts.active].inventory_overlay(batch.width,batch.height,blocked)
    }
    pub(super) fn craftix_inventory_touch(&mut self, touch: &Touch, size: [u32;2]) -> bool {
        self.craftix_context(true);
        self.craftix_resize(size);
        let point=[touch.location.x as f32 / size[0].max(1) as f32, touch.location.y as f32 / size[1].max(1) as f32];
        let aspect=size[0].max(1) as f32 / size[1].max(1) as f32;
        let layout=self.craftix_inventory_layout(&horizon_client::hud::HudBatch::new(size));
        let owned=self.pad.craftix.router.owns_pointer(touch.id);
        if !(owned || touch.phase == TouchPhase::Started && layout.hit_for(point,aspect,true).is_some()) { return false; }
        match touch.phase {
            TouchPhase::Started => self.pad.craftix.router.begin_for(&layout,touch.id,point,aspect,
                self.started.elapsed().as_millis().min(u64::MAX as u128) as u64,true),
            TouchPhase::Moved => self.pad.craftix.router.moved(&layout,touch.id,point,aspect),
            TouchPhase::Ended => self.pad.craftix.router.end(touch.id),
            TouchPhase::Cancelled => { self.craftix_clear(); return true; }
        }
        let events=self.pad.craftix.router.drain();
        let events=self.pad.craftix.bridge.translate(events);
        self.craftix_dispatch(events,size[0],size[1]);
        true
    }
    pub(super) fn craftix_resize(&mut self, size: [u32; 2]) {
        if self.pad.craftix.viewport.is_some_and(|old| old != size) {
            self.craftix_clear();
            if let Some(editor) = &mut self.pad.craftix.editor { editor.cancel_drag(); }
        }
        self.pad.craftix.viewport = Some(size);
    }
    pub(super) fn craftix_editor_touch(&mut self, touch: &Touch, size: [u32; 2]) {
        self.craftix_resize(size);
        if touch.phase == TouchPhase::Cancelled {
            if let Some(editor) = &mut self.pad.craftix.editor { editor.cancel_drag(); }
            return;
        }
        let point = [touch.location.x as f32 / size[0].max(1) as f32,
            touch.location.y as f32 / size[1].max(1) as f32];
        let aspect = size[0].max(1) as f32 / size[1].max(1) as f32;
        let Some(editor) = self.pad.craftix.editor.as_mut() else { return; };
        let result = editor.touch(touch.id, point, aspect, touch.phase == TouchPhase::Started, touch.phase == TouchPhase::Ended);
        match result {
            UiEvent::None => {},
            UiEvent::ShowText(initial) => {
                crate::platform::show(&initial);
                if !crate::platform::active() { editor.finish_text(); editor.error = "System keyboard unavailable".into(); }
            }
            UiEvent::Cancel => {
                log::info!("CRAFTIX controls editor closed");
                self.pad.craftix.editor = None;
                crate::platform::hide();
            }
            UiEvent::Import => {
                let result = std::env::current_dir().and_then(|home|
                    horizon_client::craftix_controls::transfer::import(&home, &mut editor.model));
                match result {
                    Ok(()) => { editor.imported(); editor.error = "Imported. Save to apply".into(); }
                    Err(error) => editor.error = format!("Import transfer-v1.json: {error}"),
                }
            }
            UiEvent::Export => {
                let result = std::env::current_dir().and_then(|home|
                    horizon_client::craftix_controls::transfer::export(&home, &editor.model));
                editor.error = match result {
                    Ok(()) => "Exported: Files / Control layouts / transfer-v1.json".into(),
                    Err(error) => format!("Export failed: {error}"),
                };
            }
            UiEvent::Save => {
                // The only current entry point to the editor is More.
                if !editor.model.staged.items.iter().all(|layout| layout.buttons.iter().any(|button|
                    button.id == horizon_client::craftix_controls::layout::Id::More && button.enabled && button.opacity > 0.0
                    && button.action == Some(horizon_client::craftix_controls::layout::Action::More))) {
                    editor.error = "Every layout needs an enabled More button".into();
                    return;
                }
                // Do not overwrite an invalid pre-existing file with defaults.
                if let Some(error) = &self.pad.craftix.load_error { editor.error = error.clone(); return; }
                // Do not activate unsaved edits when writing fails.
                match editor.model.staged.save(std::path::Path::new("craftix-controls-v1.json")) {
                    Ok(()) => {
                        log::info!("CRAFTIX controls layout saved");
                        self.pad.craftix.layouts = editor.model.staged.clone();
                        self.pad.craftix.editor = None;
                        crate::platform::hide();
                    }
                    Err(error) => editor.error = format!("Save failed: {error}"),
                }
            }
        }
    }
    pub(super) fn craftix_touch(&mut self, touch: &Touch, size: [u32; 2]) {
        self.craftix_context(false);
        self.craftix_resize(size);
        let point = [touch.location.x as f32 / size[0].max(1) as f32,
            touch.location.y as f32 / size[1].max(1) as f32];
        let aspect = size[0].max(1) as f32 / size[1].max(1) as f32;
        if touch.phase == TouchPhase::Cancelled { self.craftix_clear(); return; }
        if self.pad.craftix.reserved.contains(&touch.id) {
            if touch.phase == TouchPhase::Ended { self.pad.craftix.reserved.remove(&touch.id); }
            return;
        }
        let batch = horizon_client::hud::HudBatch::new(size);
        let layout = self.pad.craftix.layouts.items[self.pad.craftix.layouts.active].resolved(batch.width,batch.height);
        if touch.phase == TouchPhase::Started {
            let p = [touch.location.x as f32 / batch.scale, touch.location.y as f32 / batch.scale];
            if layout.hit(point, aspect).is_none()
                && let Some(slot) = horizon_client::touch::hotbar_slot(batch.width, batch.height, p)
            {
                self.selected = slot;
                self.pad.craftix.reserved.insert(touch.id);
                return;
            }
        }
        match touch.phase {
            TouchPhase::Started => self.pad.craftix.router.begin(&layout, touch.id, point, aspect,
                self.started.elapsed().as_millis().min(u64::MAX as u128) as u64),
            TouchPhase::Moved => self.pad.craftix.router.moved(&layout, touch.id, point, aspect),
            TouchPhase::Ended => self.pad.craftix.router.end(touch.id),
            TouchPhase::Cancelled => unreachable!(),
        }
        let events = self.pad.craftix.router.drain();
        let events = self.pad.craftix.bridge.translate(events);
        self.pad.in_use = true;
        self.craftix_dispatch(events, size[0], size[1]);
    }
}
