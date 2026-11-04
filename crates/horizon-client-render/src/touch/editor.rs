use super::*;

pub struct Editor {
    pub selected: Option<Control>,
    option: Option<usize>,
    apply_all: bool,
    original: Settings,
    drag: Option<(u64, Drag)>,
    error: Option<String>,
    palette: bool,
    selected_custom: Option<usize>,
    key_picker: bool,
    key_page: usize,
    raw_keys: bool,
    name_editing: bool,
    name_request: bool,
    layout_menu: bool,
    layout_request: Option<LayoutRequest>,
    manager: bool,
    return_to_manager: bool,
    manager_page: usize,
    manager_slider: Option<(u64, usize)>,
    layout_slot: u8,
    confirm_reset: bool,
    opened_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutRequest { Import, Export, SaveSlot(u8), LoadSlot(u8) }

enum Drag {
    Control([f32; 2]),
    Slider(usize),
}

impl TouchControls {
    pub(super) fn cancel_editor_drag(&mut self) {
        if let Some(e) = &mut self.editor {
            e.drag = None;
            e.manager_slider = None;
        }
    }
    pub fn open_editor(&mut self) {
        self.clear();
        self.editor = Some(Editor {
            selected: None,
            option: None,
            apply_all: false,
            original: self.settings.clone(),
            drag: None,
            error: None,
            palette: false,
            selected_custom: None,
            key_picker: false,
            key_page: 0,
            raw_keys: false,
            name_editing: false,
            name_request: false,
            layout_menu: false,
            layout_request: None,
            manager: false,
            return_to_manager: false,
            manager_page: 0,
            manager_slider: None,
            layout_slot: 1,
            confirm_reset: false,
            opened_at: Instant::now(),
        });
    }
    pub fn open_custom_manager(&mut self) {
        self.open_editor();
        if let Some(e) = &mut self.editor {
            e.manager = true;
            if !self.settings.custom_buttons.is_empty() { e.selected_custom = Some(0); }
        }
    }
    pub fn take_layout_request(&mut self) -> Option<LayoutRequest> {
        self.editor.as_mut()?.layout_request.take()
    }
    pub fn layout_status(&mut self, message: String) {
        if let Some(e) = &mut self.editor { e.error = Some(message); }
    }
    /// A host displays its real text input when a name field is tapped.
    pub fn take_custom_name_request(&mut self) -> Option<String> {
        let e = self.editor.as_mut()?;
        if !std::mem::take(&mut e.name_request) { return None; }
        let i = e.selected_custom?;
        Some(self.settings.custom_buttons.get(i)?.name.clone())
    }
    pub fn custom_name_editing(&self) -> bool {
        self.editor.as_ref().is_some_and(|e| e.name_editing)
    }
    pub fn custom_name_text(&mut self, text: &str) {
        if let Some(i) = self.editor.as_ref().filter(|e| e.name_editing).and_then(|e| e.selected_custom)
            && let Some(button) = self.settings.custom_buttons.get_mut(i)
        {
            // Keep the IME and model in sync until editing ends; truncating here
            // would make later backspaces delete the wrong visible characters.
            button.name.push_str(text);
        }
    }
    pub fn custom_name_backspace(&mut self) {
        if let Some(i) = self.editor.as_ref().filter(|e| e.name_editing).and_then(|e| e.selected_custom)
            && let Some(button) = self.settings.custom_buttons.get_mut(i)
        {
            button.name.pop();
        }
    }
    pub fn finish_custom_name(&mut self) {
        if let Some(i) = self.editor.as_ref().and_then(|e| e.selected_custom)
            && let Some(button) = self.settings.custom_buttons.get_mut(i)
        {
            button.name = button.name.chars().filter(|c| !c.is_control()).take(32).collect();
        }
        if let Some(e) = &mut self.editor {
            e.name_editing = false;
            e.name_request = false;
        }
    }
    pub fn cancel_editor(&mut self) {
        if let Some(e) = self.editor.take() {
            self.settings = e.original;
        }
        self.clear();
    }
    /// The editor owns every finger: editing cannot move, mine or open menus.
    /// Returns true when Done/Cancel closed the editor.
    pub fn edit_touch(&mut self, id: u64, p: [f32; 2], started: bool, ended: bool) -> bool {
        let Some(mut e) = self.editor.take() else {
            return false;
        };
        self.clicked |= started;
        // While the system IME owns this edit, do not change the selected
        // button or activate editor commands under the keyboard.
        if e.name_editing {
            self.editor = Some(e);
            return false;
        }
        if e.layout_menu {
            if started {
                if !layout::contains([0.28, 0.62, 0.44, 0.07], p) {
                    e.confirm_reset = false;
                }
                if layout::contains([0.28, 0.26, 0.2, 0.08], p) {
                    self.settings.apply_control_preset(false);
                    e.error = Some("Standard preset applied. Tap Done to save.".into());
                } else if layout::contains([0.52, 0.26, 0.2, 0.08], p) {
                    self.settings.apply_control_preset(true);
                    e.error = Some("Left-handed preset applied. Tap Done to save.".into());
                } else if cfg!(target_os = "android") && layout::contains([0.28, 0.4, 0.2, 0.08], p) {
                    e.layout_request = Some(LayoutRequest::Export);
                } else if cfg!(target_os = "android") && layout::contains([0.52, 0.4, 0.2, 0.08], p) {
                    e.layout_request = Some(LayoutRequest::Import);
                } else if cfg!(target_os = "android") && layout::contains([0.28, 0.50, 0.13, 0.08], p) {
                    e.layout_slot = e.layout_slot % 3 + 1;
                } else if cfg!(target_os = "android") && layout::contains([0.42, 0.50, 0.13, 0.08], p) {
                    e.layout_request = Some(LayoutRequest::SaveSlot(e.layout_slot));
                } else if cfg!(target_os = "android") && layout::contains([0.56, 0.50, 0.16, 0.08], p) {
                    e.layout_request = Some(LayoutRequest::LoadSlot(e.layout_slot));
                } else if layout::contains([0.28, 0.62, 0.44, 0.07], p) {
                    if e.confirm_reset {
                        self.settings.apply_control_preset(false);
                        self.settings.custom_buttons.clear();
                        e.confirm_reset = false;
                        e.error = Some("Defaults restored. Tap Done to save.".into());
                    } else {
                        e.confirm_reset = true;
                        e.error = Some("Tap Confirm reset to remove custom buttons.".into());
                    }
                } else if layout::contains([0.4, 0.71, 0.2, 0.07], p) {
                    e.layout_menu = false;
                    e.confirm_reset = false;
                    e.manager = e.return_to_manager;
                    e.return_to_manager = false;
                    e.error = None;
                }
            }
            self.editor = Some(e);
            return false;
        }
        if e.manager {
            if e.key_picker {
                self.edit_custom(&mut e, id, p, started, ended);
                self.editor = Some(e);
                return false;
            }
            if self.manager_touch(&mut e, id, p, started, ended) {
                self.clear();
                return true;
            }
            self.editor = Some(e);
            return false;
        }
        if self.edit_custom(&mut e, id, p, started, ended) {
            self.editor = Some(e);
            return false;
        }
        if started {
            if layout::contains([0.02, 0.04, 0.105, 0.075], p) {
                match self.settings.save() {
                    Ok(()) => {
                        self.clear();
                        return true;
                    }
                    Err(error) => e.error = Some(error.to_string()),
                }
            } else if layout::contains([0.135, 0.04, 0.105, 0.075], p) {
                self.settings = e.original;
                self.clear();
                return true;
            } else if layout::contains([0.25, 0.04, 0.105, 0.075], p) {
                self.settings.layouts[self.settings.mode.index()] =
                    Settings::default().layouts[self.settings.mode.index()];
                self.settings.button_visibility = [0; 14];
                for (i, button) in self.settings.custom_buttons.iter_mut().enumerate() {
                    button.layouts[self.settings.mode.index()] =
                        CustomButton::new(button.key.clone(), i).layouts
                            [self.settings.mode.index()];
                }
            } else if layout::contains([0.37, 0.04, 0.105, 0.075], p) {
                e.layout_menu = true;
                e.selected = None;
                e.selected_custom = None;
                e.palette = false;
                e.key_picker = false;
                e.drag = None;
                e.error = None;
            } else if layout::contains([0.49, 0.04, 0.105, 0.075], p) {
                e.manager = true;
                e.palette = false;
                e.key_picker = false;
                e.selected = None;
                e.drag = None;
            } else if layout::contains([0.02, 0.13, 0.335, 0.075], p) {
                e.palette = !e.palette;
                e.selected = None;
                e.selected_custom = None;
                e.key_picker = false;
                e.drag = None;
            } else if e.palette {
                for (index, control) in Control::ALL.into_iter().enumerate() {
                    if layout::contains(palette_rect(index), p) {
                        let show = !self.visible(control);
                        self.settings.button_visibility[index] = if show { 1 } else { 2 };
                    }
                }
            } else if layout::contains([0.32, 0.88, 0.36, 0.07], p) && e.selected.is_some() {
                self.settings.button_visibility[e.selected.unwrap() as usize] = 2;
                e.selected = None;
                e.drag = None;
            } else if layout::contains([0.32, 0.61, 0.36, 0.06], p)
                && e.selected.is_some()
                && e.option.is_some()
            {
                e.apply_all = !e.apply_all;
            } else if layout::contains([0.32, 0.68, 0.36, 0.07], p)
                && e.selected.is_some()
                && e.option.is_some()
            {
                e.drag = Some((id, Drag::Slider(e.option.unwrap())));
            } else if layout::contains([0.32, 0.79, 0.17, 0.07], p) && e.selected.is_some() {
                e.option = Some(1);
            } else if layout::contains([0.51, 0.79, 0.17, 0.07], p) && e.selected.is_some() {
                e.option = Some(0);
            } else if let Some(c) = Control::ALL.into_iter().rev().find(|c| {
                self.visible(*c) && layout::contains(self.placement(*c).rect(self.aspect()), p)
            }) {
                e.selected = Some(c);
                e.option = None;
                let center = self.placement(c).center;
                e.drag = Some((id, Drag::Control([p[0] - center[0], p[1] - center[1]])));
            } else {
                e.selected = None;
                e.option = None;
            }
        }
        if let Some((owner, drag)) = &e.drag
            && *owner == id
            && let Some(selected) = e.selected
        {
            let aspect = self.aspect();
            let slot = self.settings.slot(selected);
            let place = &mut self.settings.layouts[self.settings.mode.index()][slot];
            match drag {
                Drag::Control(offset) => {
                    let mut center = [p[0] - offset[0], p[1] - offset[1]];
                    if self.settings.left_handed {
                        center[0] = 1.0 - center[0];
                    }
                    // Match the displayed size on narrow portrait surfaces.
                    // A stored button may be wider than the screen; its raw
                    // size would otherwise produce inverted clamp bounds.
                    let size = (place.size * self.settings.button_scale)
                        .min(aspect)
                        .min(1.0);
                    place.center = [
                        center[0].clamp(size / aspect / 2.0, 1.0 - size / aspect / 2.0),
                        center[1].clamp(size / 2.0, 1.0 - size / 2.0),
                    ];
                }
                Drag::Slider(index) => {
                    let value = ((p[0] - 0.335) / 0.33).clamp(0.0, 1.0);
                    for (i, place) in self.settings.layouts[self.settings.mode.index()]
                        .iter_mut()
                        .enumerate()
                    {
                        if i == slot || e.apply_all {
                            if *index == 0 {
                                place.size = 0.045 + value * 0.405;
                            } else {
                                place.opacity = value;
                            }
                        }
                    }
                }
            }
            if ended {
                e.drag = None;
            }
        }

        self.editor = Some(e);
        false
    }
    fn manager_touch(&mut self, e: &mut Editor, id: u64, p: [f32; 2], started: bool, ended: bool) -> bool {
        if started {
            if layout::contains([0.70, 0.13, 0.09, 0.06], p) {
                match self.settings.save() {
                    Ok(()) => return true,
                    Err(error) => e.error = Some(format!("Save failed: {error}")),
                }
            } else if layout::contains([0.60, 0.13, 0.09, 0.06], p) {
                self.settings = e.original.clone();
                return true;
            } else if layout::contains([0.20, 0.29, 0.12, 0.07], p) {
                e.manager = false;
                e.return_to_manager = true;
                e.selected_custom = None;
                e.layout_menu = true;
                e.error = None;
            } else if layout::contains([0.20, 0.38, 0.12, 0.07], p) {
                e.manager = false;
                e.drag = None;
            } else if layout::contains([0.34, 0.60, 0.20, 0.06], p) {
                if self.settings.custom_buttons.len() < 32 {
                    let i = self.settings.custom_buttons.len();
                    self.settings.custom_buttons.push(CustomButton::new("key.keyboard.unknown".into(), i));
                    e.selected_custom = Some(i);
                    e.manager_page = i / 4;
                    e.raw_keys = true;
                    e.key_page = 0;
                    e.key_picker = true;
                } else { e.error = Some("Maximum: 32 buttons".into()); }
            } else if layout::contains([0.34, 0.53, 0.095, 0.05], p) {
                e.manager_page = e.manager_page.saturating_sub(1);
            } else if layout::contains([0.445, 0.53, 0.095, 0.05], p) {
                e.manager_page = (e.manager_page + 1).min(self.settings.custom_buttons.len().saturating_sub(1) / 4);
            } else {
                for row in 0..4 {
                    let i = e.manager_page * 4 + row;
                    if i < self.settings.custom_buttons.len() && layout::contains(manager_row(row), p) {
                        e.selected_custom = Some(i);
                        e.error = None;
                        break;
                    }
                }
                if let Some(i) = e.selected_custom {
                    if layout::contains([0.56, 0.22, 0.23, 0.055], p) {
                        e.name_editing = true; e.name_request = true;
                    } else if layout::contains([0.56, 0.29, 0.11, 0.055], p) {
                        self.settings.custom_buttons[i].icon = self.settings.custom_buttons[i].icon.next();
                    } else if layout::contains([0.68, 0.29, 0.11, 0.055], p) {
                        self.settings.custom_buttons[i].enabled ^= true;
                    } else if layout::contains([0.56, 0.36, 0.23, 0.055], p) {
                        e.key_picker = true; e.raw_keys = true; e.key_page = 0;
                    } else if layout::contains([0.56, 0.435, 0.23, 0.055], p) {
                        e.manager_slider = Some((id, 0));
                    } else if layout::contains([0.56, 0.505, 0.23, 0.055], p) {
                        e.manager_slider = Some((id, 1));
                    } else if layout::contains([0.56, 0.565, 0.11, 0.045], p) {
                        let b = &mut self.settings.custom_buttons[i];
                        b.visibility = b.visibility.next();
                    } else if layout::contains([0.68, 0.565, 0.11, 0.045], p) {
                        self.settings.custom_buttons[i].toggle ^= true;
                    } else if layout::contains([0.56, 0.62, 0.11, 0.04], p) {
                        self.settings.custom_buttons.remove(i);
                        e.selected_custom = None;
                        e.manager_page = e.manager_page.min(self.settings.custom_buttons.len().saturating_sub(1) / 4);
                    } else if layout::contains([0.68, 0.62, 0.11, 0.04], p) {
                        e.manager = false; e.drag = None;
                    }
                }
            }
        }
        if let Some((owner, slider)) = e.manager_slider && owner == id {
            if let Some(i) = e.selected_custom && let Some(button) = self.settings.custom_buttons.get_mut(i) {
                let value = ((p[0] - 0.56) / 0.23).clamp(0.0, 1.0);
                let placement = &mut button.layouts[self.settings.mode.index()];
                if slider == 0 { placement.size = 0.045 + value * 0.405; }
                else { placement.opacity = value; }
            }
            if ended { e.manager_slider = None; }
        }
        false
    }
    fn draw_manager(&self, b: &mut HudBatch, a: &HudAtlas, e: &Editor) {
        glass(b, a, [0.18, 0.10, 0.64, 0.59], false);
        glass(b, a, [0.19, 0.21, 0.14, 0.46], false);
        glass(b, a, [0.335, 0.21, 0.21, 0.46], false);
        glass(b, a, [0.55, 0.21, 0.25, 0.46], false);
        manager_text(b, a, [0.20, 0.12, 0.36, 0.07], "MORE  /  CUSTOM CONTROLS");
        manager_plate(b, a, [0.60, 0.13, 0.09, 0.06], "Cancel", false);
        manager_plate(b, a, [0.70, 0.13, 0.09, 0.06], "Save", true);
        manager_plate(b, a, [0.20, 0.22, 0.12, 0.06], "Buttons", true);
        manager_plate(b, a, [0.20, 0.29, 0.12, 0.07], "Layouts", false);
        manager_plate(b, a, [0.20, 0.38, 0.12, 0.07], "Position", false);
        manager_text(b, a, [0.34, 0.215, 0.20, 0.05], &format!("Buttons ({})", self.settings.custom_buttons.len()));
        for (row, (i, button)) in self.settings.custom_buttons.iter().enumerate().skip(e.manager_page * 4).take(4).enumerate() {
            let text = format!("{}  /  {}{}", button.label(), button.binding_label(), if button.enabled { "" } else { " (off)" });
            manager_plate(b, a, manager_row(row), &text, e.selected_custom == Some(i));
        }
        manager_plate(b, a, [0.34, 0.53, 0.095, 0.05], "< Prev", false);
        manager_plate(b, a, [0.445, 0.53, 0.095, 0.05], "Next >", false);
        manager_plate(b, a, [0.34, 0.60, 0.20, 0.06], "+ Add button", true);
        if let Some(i) = e.selected_custom && let Some(button) = self.settings.custom_buttons.get(i) {
            let p = button.layouts[self.settings.mode.index()];
            manager_plate(b, a, [0.56, 0.22, 0.23, 0.055], &format!("Name: {}", button.label()), false);
            manager_plate(b, a, [0.56, 0.29, 0.11, 0.055], &format!("Icon: {}", button.icon.label()), false);
            manager_plate(b, a, [0.68, 0.29, 0.11, 0.055], if button.enabled { "Enabled" } else { "Disabled" }, button.enabled);
            manager_plate(b, a, [0.56, 0.36, 0.23, 0.055], &format!("Key: {}", button.binding_label()), false);
            manager_plate(b, a, [0.56, 0.435, 0.23, 0.055], &format!("Size: {:.0}%", p.size * 100.0), false);
            manager_plate(b, a, [0.56, 0.505, 0.23, 0.055], &format!("Opacity: {:.0}%", p.opacity * 100.0), false);
            manager_slider(b, a, [0.56, 0.435, 0.23, 0.055], ((p.size - 0.045) / 0.405).clamp(0.0, 1.0));
            manager_slider(b, a, [0.56, 0.505, 0.23, 0.055], p.opacity.clamp(0.0, 1.0));
            manager_plate(b, a, [0.56, 0.565, 0.11, 0.045], button.visibility.label(), false);
            manager_plate(b, a, [0.68, 0.565, 0.11, 0.045], if button.toggle { "Toggle" } else { "Hold" }, button.toggle);
            manager_plate(b, a, [0.56, 0.62, 0.11, 0.04], "Delete", false);
            manager_plate(b, a, [0.68, 0.62, 0.11, 0.04], "Move", false);
        } else {
            manager_text(b, a, [0.56, 0.30, 0.23, 0.09], "Select or add a button");
        }
        if let Some(error) = &e.error { manager_text(b, a, [0.20, 0.665, 0.60, 0.02], error); }
    }
    pub(super) fn draw_editor_toolbar(&self, b: &mut HudBatch, a: &HudAtlas) {
        if self.editor.as_ref().is_some_and(|e| e.manager) { return; }
        for (rect, label) in [
            ([0.02, 0.04, 0.105, 0.075], "Done"),
            ([0.135, 0.04, 0.105, 0.075], "Cancel"),
            ([0.25, 0.04, 0.105, 0.075], "Reset"),
            ([0.37, 0.04, 0.105, 0.075], "Layouts"),
            ([0.49, 0.04, 0.105, 0.075], "Custom"),
            ([0.02, 0.13, 0.335, 0.075], "Add buttons..."),
        ] {
            plate(b, a, rect, label, false);
        }
    }

    pub(super) fn draw_editor(&self, b: &mut HudBatch, a: &HudAtlas) {
        let e = self.editor.as_ref().unwrap();
        let (w, h) = (b.width, b.height);
        let text = rgba(245, 245, 245, 255);
        if e.manager {
            let start = b.vertices.len();
            self.draw_manager(b, a, e);
            if e.key_picker { self.draw_custom_editor(b, a, e); }
            let t = (e.opened_at.elapsed().as_secs_f32() / 0.18).clamp(0.0, 1.0);
            let opacity = t * t * (3.0 - 2.0 * t);
            for vertex in &mut b.vertices[start..] {
                let alpha = ((vertex.color >> 24) as f32 * opacity).round() as u32;
                vertex.color = (vertex.color & 0x00ff_ffff) | (alpha << 24);
            }
            return;
        }
        if e.layout_menu {
            b.fill(a, w * 0.24, h * 0.19, w * 0.52, h * 0.70, rgba(15, 10, 25, 245));
            plate(b, a, [0.28, 0.26, 0.2, 0.08], "Standard", false);
            plate(b, a, [0.52, 0.26, 0.2, 0.08], "Left-handed", false);
            if cfg!(target_os = "android") {
                plate(b, a, [0.28, 0.4, 0.2, 0.08], "Export", false);
                plate(b, a, [0.52, 0.4, 0.2, 0.08], "Import", false);
                plate(b, a, [0.28, 0.50, 0.13, 0.08], &format!("Slot {}", e.layout_slot), false);
                plate(b, a, [0.42, 0.50, 0.13, 0.08], "Save", false);
                plate(b, a, [0.56, 0.50, 0.16, 0.08], "Load", false);
            }
            plate(b, a, [0.28, 0.62, 0.44, 0.07],
                if e.confirm_reset { "Confirm reset (removes custom buttons)" } else { "Restore defaults..." }, e.confirm_reset);
            plate(b, a, [0.4, 0.71, 0.2, 0.07], "Back", false);
            if let Some(message) = &e.error {
                b.text(a, message, w * 0.26, h * 0.82, text, true);
            }
            return;
        }
        if let Some(selected) = e.selected {
            plate(b, a, [0.32, 0.88, 0.36, 0.07], "Remove button", false);
            let p = self.placement(selected);
            plate(
                b,
                a,
                [0.32, 0.79, 0.17, 0.07],
                "Opacity",
                e.option == Some(1),
            );
            plate(b, a, [0.51, 0.79, 0.17, 0.07], "Size", e.option == Some(0));
            if let Some(option) = e.option {
                b.fill(
                    a,
                    w * 0.31,
                    h * 0.59,
                    w * 0.38,
                    h * 0.18,
                    rgba(15, 15, 15, 235),
                );
                plate(
                    b,
                    a,
                    [0.32, 0.61, 0.36, 0.06],
                    if e.apply_all {
                        "Apply to all: On"
                    } else {
                        "Apply to all: Off"
                    },
                    e.apply_all,
                );
                let value = if option == 0 {
                    (p.size - 0.045) / 0.405
                } else {
                    p.opacity
                };
                let percent = if option == 0 {
                    p.size * 100.0
                } else {
                    p.opacity * 100.0
                };
                b.text(
                    a,
                    &format!("{} {:.0}%", selected.name(), percent),
                    w * 0.325,
                    h * 0.68,
                    text,
                    true,
                );
                nine(
                    b,
                    a,
                    "ui/slider_background",
                    1.0,
                    [w * 0.335, h * 0.72, w * 0.33, h * 0.025],
                    rgba(255, 255, 255, 255),
                );
                nine(
                    b,
                    a,
                    "ui/slider_button_default",
                    2.0,
                    [w * (0.335 + value * 0.33) - 3.0, h * 0.707, 6.0, h * 0.05],
                    rgba(255, 255, 255, 255),
                );
            }
        }
        if e.palette {
            b.fill(
                a,
                w * 0.2,
                h * 0.14,
                w * 0.6,
                h * 0.72,
                rgba(15, 15, 15, 245),
            );
            for (index, control) in Control::ALL.into_iter().enumerate() {
                let shown = self.visible(control);
                plate(
                    b,
                    a,
                    palette_rect(index),
                    &format!("{} {}", if shown { "[-]" } else { "[+]" }, control.name()),
                    shown,
                );
            }
        }
        self.draw_custom_editor(b, a, e);
        if let Some(error) = &e.error {
            b.text(
                a,
                error,
                w * 0.33,
                h * 0.88,
                rgba(255, 120, 120, 255),
                true,
            );
        }
    }
}
fn manager_row(row: usize) -> [f32; 4] { [0.34, 0.27 + row as f32 * 0.06, 0.20, 0.055] }
pub(super) fn glass(b: &mut HudBatch, a: &HudAtlas, [x,y,w,h]: [f32; 4], on: bool) {
    let (x,y,w,h) = (x*b.width,y*b.height,w*b.width,h*b.height);
    let radius = 4.0_f32.min(w * 0.2).min(h * 0.3);
    let fill = if on { rgba(92,30,144,220) } else { rgba(12,9,26,225) };
    // A small chamfer approximates rounded dark glass using constant geometry.
    b.fill(a, x + radius, y, w - radius * 2.0, h, fill);
    b.fill(a, x, y + radius, radius, h - radius * 2.0, fill);
    b.fill(a, x + w - radius, y + radius, radius, h - radius * 2.0, fill);
    for tri in [
        [[x+radius,y],[x+radius,y+radius],[x,y+radius]],
        [[x+w-radius,y],[x+w,y+radius],[x+w-radius,y+radius]],
        [[x,y+h-radius],[x+radius,y+h-radius],[x+radius,y+h]],
        [[x+w-radius,y+h-radius],[x+w,y+h-radius],[x+w-radius,y+h]],
    ] { b.fill_quad(a, [tri[0],tri[1],tri[2],tri[2]], fill); }
    let color = if on { rgba(211,145,255,255) } else { rgba(150,90,205,210) };
    let points = [[x+radius,y],[x+w-radius,y],[x+w,y+radius],[x+w,y+h-radius],
        [x+w-radius,y+h],[x+radius,y+h],[x,y+h-radius],[x,y+radius],[x+radius,y]];
    for pair in points.windows(2) { line(b,a,pair[0],pair[1],color); }
    if on {
        line(b,a,[x+radius,y+1.0],[x+w-radius,y+1.0],rgba(208,120,255,65));
        line(b,a,[x+radius,y+h-1.0],[x+w-radius,y+h-1.0],rgba(208,120,255,65));
    }
}
fn manager_text(b: &mut HudBatch, a: &HudAtlas, [x,y,w,h]: [f32;4], text: &str) {
    let (x,y,w,h)=(x*b.width,y*b.height,w*b.width,h*b.height);
    let mut label = text.to_owned();
    while HudBatch::text_width(a, &label) > (w-6.0).max(1.0) && !label.is_empty() { label.pop(); }
    b.text(a,&label,x+3.0,y+(h-8.0)/2.0,rgba(245,240,255,255),false);
}
fn manager_slider(b: &mut HudBatch, a: &HudAtlas, [x,y,w,h]: [f32;4], value: f32) {
    let (x,y,w,h) = (x*b.width,y*b.height,w*b.width,h*b.height);
    let track = (w - 8.0).max(0.0);
    b.fill(a, x+4.0, y+h-3.0, track, 1.0, rgba(95,65,120,220));
    b.fill(a, x+4.0, y+h-3.0, track*value, 1.0, rgba(200,125,255,255));
    b.fill(a, x+3.0+track*value, y+h-4.0, 3.0, 3.0, rgba(245,220,255,255));
}
fn manager_plate(b: &mut HudBatch, a: &HudAtlas, rect: [f32;4], text: &str, on: bool) {
    glass(b,a,rect,on); manager_text(b,a,rect,text);
}
fn palette_rect(index: usize) -> [f32; 4] {
    [
        0.22 + (index % 2) as f32 * 0.29,
        0.17 + (index / 2) as f32 * 0.11,
        0.27,
        0.09,
    ]
}
fn plate(b: &mut HudBatch, a: &HudAtlas, [x, y, w, h]: [f32; 4], label: &str, on: bool) {
    let (bw, bh) = (b.width, b.height);
    nine(
        b,
        a,
        "ui/button_borderless_light",
        1.0,
        [x * bw, y * bh, w * bw, h * bh],
        if on {
            rgba(100, 210, 90, 255)
        } else {
            rgba(255, 255, 255, 255)
        },
    );
    if let Some((first, second)) = label.split_once(" & ") {
        b.text(
            a,
            &format!("{first} &"),
            x * bw + 3.0,
            y * bh + (h * bh - 20.0) / 2.0,
            rgba(35, 35, 35, 255),
            false,
        );
        b.text(
            a,
            second,
            x * bw + 3.0,
            y * bh + (h * bh - 20.0) / 2.0 + 10.0,
            rgba(35, 35, 35, 255),
            false,
        );
    } else {
        let mut display = label.to_owned();
        while HudBatch::text_width(a, &display) > w * bw - 6.0 && !display.is_empty() {
            display.pop();
        }
        b.text(
            a,
            &display,
            x * bw + 3.0,
            y * bh + (h * bh - 8.0) / 2.0,
            rgba(35, 35, 35, 255),
            false,
        );
    }
}

impl TouchControls {
    fn edit_custom(
        &mut self,
        e: &mut Editor,
        id: u64,
        p: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if p[1] < 0.13 && started {
            return false;
        }
        if e.key_picker {
            if started {
                if layout::contains([0.15, 0.16, 0.33, 0.06], p) {
                    e.raw_keys = false;
                    e.key_page = 0;
                } else if layout::contains([0.52, 0.16, 0.33, 0.06], p) {
                    e.raw_keys = true;
                    e.key_page = 0;
                } else if layout::contains([0.2, 0.86, 0.18, 0.07], p) {
                    e.key_page = e.key_page.saturating_sub(1);
                } else if layout::contains([0.41, 0.86, 0.18, 0.07], p) {
                    e.key_picker = false;
                } else if layout::contains([0.62, 0.86, 0.18, 0.07], p) {
                    e.key_page = (e.key_page + 1)
                        .min(super::custom::choices(e.raw_keys).len().saturating_sub(1) / 8);
                } else {
                    for (i, (token, _)) in super::custom::choices(e.raw_keys)
                        .into_iter()
                        .skip(e.key_page * 8)
                        .take(8)
                        .enumerate()
                    {
                        if layout::contains(key_rect(i), p)
                            && let Some(index) = e.selected_custom
                        {
                            self.settings.custom_buttons[index].key = token;
                            e.key_picker = false;
                            break;
                        }
                    }
                }
            }
            return true;
        }
        if e.palette && started && layout::contains([0.22, 0.84, 0.56, 0.07], p) {
            if self.settings.custom_buttons.len() < 32 {
                let index = self.settings.custom_buttons.len();
                self.settings
                    .custom_buttons
                    .push(CustomButton::new("key.keyboard.unknown".into(), index));
                e.selected = None;
                e.selected_custom = Some(index);
                e.key_picker = true;
                e.raw_keys = false;
                e.key_page = 0;
                e.palette = false;
                e.option = None;
                e.drag = None;
            }
            return true;
        }
        if e.palette {
            return false;
        }
        if started && e.selected_custom.is_some() {
            let i = e.selected_custom.unwrap();
            if layout::contains([0.32, 0.28, 0.17, 0.07], p) && i > 0 {
                self.settings.custom_buttons.swap(i, i - 1);
                e.selected_custom = Some(i - 1);
                return true;
            }
            if layout::contains([0.51, 0.28, 0.17, 0.07], p) && i + 1 < self.settings.custom_buttons.len() {
                self.settings.custom_buttons.swap(i, i + 1);
                e.selected_custom = Some(i + 1);
                return true;
            }
            if layout::contains([0.32, 0.37, 0.36, 0.07], p) {
                e.name_editing = true;
                e.name_request = true;
                e.drag = None;
                return true;
            }
            if layout::contains([0.32, 0.45, 0.17, 0.07], p) {
                self.settings.custom_buttons[i].icon = self.settings.custom_buttons[i].icon.next();
                return true;
            }
            if layout::contains([0.51, 0.45, 0.17, 0.07], p) {
                self.settings.custom_buttons[i].enabled ^= true;
                return true;
            }
            if layout::contains([0.32, 0.54, 0.36, 0.07], p) {
                e.key_picker = true;
                e.key_page = 0;
                e.drag = None;
                return true;
            }
            if layout::contains([0.32, 0.88, 0.36, 0.07], p) {
                self.settings
                    .custom_buttons
                    .remove(e.selected_custom.unwrap());
                e.selected_custom = None;
                e.drag = None;
                return true;
            }
            if layout::contains([0.32, 0.79, 0.17, 0.07], p) {
                e.option = Some(1);
                return true;
            }
            if layout::contains([0.51, 0.79, 0.17, 0.07], p) {
                e.option = Some(0);
                return true;
            }
            if layout::contains([0.32, 0.68, 0.36, 0.07], p)
                && let Some(option) = e.option
            {
                e.drag = Some((id, Drag::Slider(option)));
            } else {
                e.selected_custom = None;
                e.drag = None;
            }
        }
        if started
            && e.drag.is_none()
            && let Some(index) = self.custom_at_with_placeholders(p, true)
        {
            let center = self.custom_placement(index).center;
            e.selected = None;
            e.selected_custom = Some(index);
            e.option = None;
            e.apply_all = false;
            e.drag = Some((id, Drag::Control([p[0] - center[0], p[1] - center[1]])));
        }
        let Some(index) = e.selected_custom else {
            return false;
        };
        if let Some((owner, drag)) = &e.drag
            && *owner == id
        {
            let aspect = self.aspect();
            let scale = self.settings.button_scale;
            let mirror = self.settings.left_handed;
            let place =
                &mut self.settings.custom_buttons[index].layouts[self.settings.mode.index()];
            match drag {
                Drag::Control(offset) => {
                    let mut center = [p[0] - offset[0], p[1] - offset[1]];
                    if mirror {
                        center[0] = 1.0 - center[0];
                    }
                    let size = (place.size * scale).min(aspect).min(1.0);
                    place.center = [
                        center[0].clamp(size / aspect / 2.0, 1.0 - size / aspect / 2.0),
                        center[1].clamp(size / 2.0, 1.0 - size / 2.0),
                    ];
                }
                Drag::Slider(option) => {
                    let value = ((p[0] - 0.335) / 0.33).clamp(0.0, 1.0);
                    if *option == 0 {
                        place.size = 0.045 + value * 0.405;
                    } else {
                        place.opacity = value;
                    }
                }
            }
            if ended {
                e.drag = None;
            }
        }
        true
    }
    pub fn capture_custom_key(&mut self, name: &str) -> bool {
        if crate::keybinds::key_code(name).is_none() {
            return false;
        }
        if let Some(e) = &mut self.editor
            && e.key_picker
            && let Some(i) = e.selected_custom
        {
            self.settings.custom_buttons[i].key = name.into();
            e.key_picker = false;
            true
        } else {
            false
        }
    }
    fn draw_custom_editor(&self, b: &mut HudBatch, a: &HudAtlas, e: &Editor) {
        let (w, h) = (b.width, b.height);
        if e.palette {
            plate(b, a, [0.22, 0.84, 0.56, 0.07], "Create button...", false);
        }
        if let Some(i) = e.selected_custom
            && !e.palette
            && !e.key_picker
        {
            let button = &self.settings.custom_buttons[i];
            let p = button.layouts[self.settings.mode.index()];
            plate(b, a, [0.32, 0.28, 0.17, 0.07], "Order back", false);
            plate(b, a, [0.51, 0.28, 0.17, 0.07], "Order front", false);
            plate(b, a, [0.32, 0.37, 0.36, 0.07],
                &format!("Name: {}", button.label()), e.name_editing);
            plate(b, a, [0.32, 0.45, 0.17, 0.07],
                &format!("Icon: {}", button.icon.label()), false);
            plate(b, a, [0.51, 0.45, 0.17, 0.07],
                if button.enabled { "Enabled" } else { "Disabled" }, button.enabled);
            plate(
                b,
                a,
                [0.32, 0.54, 0.36, 0.07],
                &format!("Binding: {}", self.settings.custom_buttons[i].binding_label()),
                false,
            );
            plate(
                b,
                a,
                [0.32, 0.79, 0.17, 0.07],
                "Opacity",
                e.option == Some(1),
            );
            plate(b, a, [0.51, 0.79, 0.17, 0.07], "Size", e.option == Some(0));
            plate(b, a, [0.32, 0.88, 0.36, 0.07], "Remove button", false);
            if let Some(option) = e.option {
                let value = if option == 0 {
                    (p.size - 0.045) / 0.405
                } else {
                    p.opacity
                };
                b.fill(
                    a,
                    w * 0.3,
                    h * 0.66,
                    w * 0.4,
                    h * 0.11,
                    rgba(15, 15, 15, 235),
                );
                let percent = if option == 0 {
                    p.size * 100.0
                } else {
                    p.opacity * 100.0
                };
                b.text(
                    a,
                    &format!(
                        "{} {:.0}%",
                        if option == 0 { "Size" } else { "Opacity" },
                        percent
                    ),
                    w * 0.325,
                    h * 0.68,
                    rgba(245, 245, 245, 255),
                    true,
                );
                nine(
                    b,
                    a,
                    "ui/slider_background",
                    1.0,
                    [w * 0.335, h * 0.72, w * 0.33, h * 0.025],
                    rgba(255, 255, 255, 255),
                );
                nine(
                    b,
                    a,
                    "ui/slider_button_default",
                    2.0,
                    [w * (0.335 + value * 0.33) - 3.0, h * 0.707, 6.0, h * 0.05],
                    rgba(255, 255, 255, 255),
                );
            }
        }
        if e.key_picker {
            b.fill(
                a,
                w * 0.12,
                h * 0.14,
                w * 0.76,
                h * 0.81,
                rgba(15, 15, 15, 250),
            );
            plate(b, a, [0.15, 0.16, 0.33, 0.06], "Game action", !e.raw_keys);
            plate(b, a, [0.52, 0.16, 0.33, 0.06], "Keyboard key", e.raw_keys);
            for (i, (_, label)) in super::custom::choices(e.raw_keys)
                .iter()
                .skip(e.key_page * 8)
                .take(8)
                .enumerate()
            {
                plate(b, a, key_rect(i), label, false);
            }
            plate(b, a, [0.2, 0.86, 0.18, 0.07], "Previous", false);
            plate(b, a, [0.41, 0.86, 0.18, 0.07], "Back", false);
            plate(b, a, [0.62, 0.86, 0.18, 0.07], "Next", false);
        }
    }
}
fn key_rect(i: usize) -> [f32; 4] {
    [0.15, 0.25 + i as f32 * 0.072, 0.7, 0.06]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_editor_updates_metadata_and_cancel_restores_it() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings.custom_buttons.push(CustomButton::new("key.keyboard.z".into(), 0));
        t.open_editor();
        let p = t.custom_placement(0).center;
        t.edit_touch(1, p, true, true);
        t.edit_touch(2, [0.40, 0.48], true, true);
        assert_eq!(t.settings.custom_buttons[0].icon, CustomIcon::Attack);
        t.edit_touch(3, [0.59, 0.48], true, true);
        assert!(!t.settings.custom_buttons[0].enabled);
        t.edit_touch(4, [0.50, 0.40], true, true);
        assert_eq!(t.take_custom_name_request(), Some(String::new()));
        t.custom_name_text("Mine");
        t.custom_name_backspace();
        assert_eq!(t.settings.custom_buttons[0].name, "Min");
        t.finish_custom_name();
        t.cancel_editor();
        assert!(t.settings.custom_buttons[0].enabled);
        assert_eq!(t.settings.custom_buttons[0].icon, CustomIcon::Text);
        assert!(t.settings.custom_buttons[0].name.is_empty());
    }

    #[test]
    fn manager_creates_real_binding_edits_and_cancels_transactionally() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.open_custom_manager();
        t.edit_touch(1, [0.42, 0.63], true, true);
        assert_eq!(t.settings.custom_buttons.len(), 1);
        assert!(t.capture_custom_key("key.keyboard.m"));
        t.edit_touch(2, [0.66, 0.245], true, true);
        assert_eq!(t.take_custom_name_request(), Some(String::new()));
        t.custom_name_text("Map");
        t.finish_custom_name();
        t.edit_touch(3, [0.75, 0.46], true, true);
        t.edit_touch(4, [0.65, 0.58], true, true);
        assert_eq!(t.settings.custom_buttons[0].name, "Map");
        assert_eq!(t.settings.custom_buttons[0].key, "key.keyboard.m");
        assert_eq!(t.settings.custom_buttons[0].visibility, CustomVisibility::Gameplay);
        assert!(t.settings.custom_buttons[0].layouts[0].size > 0.3);
        t.edit_touch(5, [0.64, 0.15], true, true);
        assert!(t.editor.is_none());
        assert!(t.settings.custom_buttons.is_empty());
    }
    #[test]
    fn manager_touches_do_not_generate_game_input() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.open_custom_manager();
        t.edit_touch(1, [0.8, 0.8], true, false);
        t.edit_touch(1, [0.9, 0.7], false, true);
        assert!(t.key_events().is_empty());
        assert_eq!(t.state().left, [0.0, 0.0]);
    }
    #[test]
    fn restore_all_requires_confirmation_and_is_reversible() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings.custom_buttons.push(CustomButton::new("key.keyboard.m".into(), 0));
        t.open_custom_manager();
        t.edit_touch(1, [0.24, 0.32], true, true);
        t.edit_touch(2, [0.50, 0.65], true, true);
        assert_eq!(t.settings.custom_buttons.len(), 1);
        t.edit_touch(3, [0.50, 0.65], true, true);
        assert!(t.settings.custom_buttons.is_empty());
        t.cancel_editor();
        assert_eq!(t.settings.custom_buttons.len(), 1);
    }
    #[test]
    fn presets_are_editor_transactions() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.open_editor();
        t.edit_touch(1, [0.42, 0.08], true, true);
        t.edit_touch(2, [0.61, 0.30], true, true);
        assert!(t.settings.left_handed);
        t.cancel_editor();
        assert!(!t.settings.left_handed);
    }
}
