//! The replacement custom-controls editor: pure widget layout, painting and
//! hit testing. The host feeds it pointers and applies the returned commands.
//!
//! It is drawn entirely from vector primitives (no legacy button sprites) so it
//! matches the round glass/neon HUD, and its geometry is plain data so the
//! whole UI can be unit tested without a window.
use super::editor::Editor;
use super::layout::{Behavior, Icon, Id};
use crate::hud::{HudAtlas, HudBatch, rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Buttons,
    Layouts,
    Position,
}
impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Buttons, Tab::Layouts, Tab::Position];
    pub fn label(self) -> &'static str {
        match self {
            Tab::Buttons => "Buttons",
            Tab::Layouts => "Layouts",
            Tab::Position => "Position",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Tab(Tab),
    /// Select a button row.
    Select(Id),
    /// Add a button by capturing a physical key.
    AddKey,
    /// Open the binding picker for the selected button.
    Picker,
    Pick(usize),
    PickerPage(bool),
    PickerRaw,
    ClosePicker,
    Rename,
    CycleIcon,
    CycleBehavior,
    Size(bool),
    Opacity(bool),
    ToggleEnabled,
    CycleVisibility,
    DeleteButton,
    Front,
    Layout(usize),
    NewLayout,
    CopyLayout,
    RenameLayout,
    DeleteLayout,
    MakeDefault,
    Mirror,
    Factory,
    Export,
    Import,
    Save,
    Cancel,
    Dismiss,
}

#[derive(Clone, Debug)]
pub struct Widget {
    pub act: Act,
    pub rect: [f32; 4],
    pub label: String,
    pub active: bool,
    pub danger: bool,
    pub enabled: bool,
}

/// Rows of picker choices offered at once.
pub const PICKER_ROWS: usize = 8;

/// What a system-IME text field is editing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTarget {
    Button(Id),
    Layout,
}

#[derive(Clone, Debug)]
pub struct EditorUi {
    pub tab: Tab,
    pub selected: Option<Id>,
    pub message: Option<String>,
    /// The next physical key press becomes a new button.
    pub capturing: bool,
    /// The key picker is open for `selected`.
    pub picker: bool,
    pub picker_raw: bool,
    pub picker_page: usize,
    /// A host text field owns the input until it finishes.
    pub text: Option<TextTarget>,
    pub confirm_delete: bool,
}
impl Default for EditorUi {
    fn default() -> Self {
        Self {
            tab: Tab::Buttons,
            selected: None,
            message: None,
            capturing: false,
            picker: false,
            picker_raw: false,
            picker_page: 0,
            text: None,
            confirm_delete: false,
        }
    }
}
impl EditorUi {
    pub fn status(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }
    /// Engine bindings first, physical keys on the second page of choices.
    pub fn choices(raw: bool) -> Vec<(String, String)> {
        if raw {
            crate::keybinds::available_keys()
                .iter()
                .map(|(name, _)| ((*name).to_string(), crate::keybinds::display_name(name).plain()))
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
    pub fn page_count(raw: bool) -> usize {
        Self::choices(raw).len().div_ceil(PICKER_ROWS).max(1)
    }
    fn area(w: f32, h: f32) -> [f32; 4] {
        let pad = (h * 0.014).max(3.0);
        [pad, pad, w - 2.0 * pad, h - 2.0 * pad]
    }
    fn row_height(h: f32) -> f32 {
        (h * 0.072).max(20.0)
    }
    fn gap(h: f32) -> f32 {
        (h * 0.012).max(2.0)
    }

    /// Every clickable region, in painting order. `hit` reads the same list.
    pub fn widgets(&self, ed: &Editor, w: f32, h: f32) -> Vec<Widget> {
        let mut out = Vec::new();
        if !w.is_finite() || !h.is_finite() || w < 32.0 || h < 32.0 {
            return out;
        }
        let [x, y, aw, ah] = Self::area(w, h);
        let gap = Self::gap(h);
        let row = Self::row_height(h);
        // Tab bar plus the two actions that close the session.
        let tab_w = (aw * 0.17).min(h * 0.62).max(30.0);
        for (i, tab) in Tab::ALL.into_iter().enumerate() {
            out.push(Widget {
                act: Act::Tab(tab),
                rect: [x + i as f32 * (tab_w + gap), y, tab_w, row],
                label: tab.label().into(),
                active: self.tab == tab,
                danger: false,
                enabled: true,
            });
        }
        let save_w = (aw * 0.13).min(h * 0.52).max(28.0);
        out.push(Widget {
            act: Act::Save,
            rect: [x + aw - 2.0 * save_w - gap, y, save_w, row],
            label: "Save".into(),
            active: false,
            danger: false,
            enabled: true,
        });
        out.push(Widget {
            act: Act::Cancel,
            rect: [x + aw - save_w, y, save_w, row],
            label: "Cancel".into(),
            active: false,
            danger: true,
            enabled: true,
        });
        if self.picker {
            self.picker_widgets(&mut out, x, y + row + gap, aw, ah - row - gap, gap, row);
            return out;
        }
        match self.tab {
            Tab::Buttons => self.button_widgets(ed, &mut out, x, y + row + gap, aw, ah - row - gap, gap, row),
            Tab::Layouts => self.layout_widgets(ed, &mut out, x, y + row + gap, aw, ah - row - gap, gap, row),
            Tab::Position => self.position_widgets(&mut out, x, y + row + gap, aw, ah - row - gap, gap, row),
        }
        out
    }

    fn button_widgets(
        &self,
        ed: &Editor,
        out: &mut Vec<Widget>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        gap: f32,
        row: f32,
    ) {
        let buttons = ed.buttons();
        let per_row = 4.0;
        let chip_w = (w - gap * (per_row - 1.0)) / per_row;
        let mut cursor = y;
        for (i, button) in buttons.iter().enumerate() {
            let col = i as f32 % per_row;
            let line = (i as f32 / per_row).floor();
            out.push(Widget {
                act: Act::Select(button.id),
                rect: [x + col * (chip_w + gap), cursor + line * (row + gap), chip_w, row],
                label: if button.enabled {
                    button.label_text()
                } else {
                    format!("{} (off)", button.label_text())
                },
                active: self.selected == Some(button.id),
                danger: false,
                enabled: button.enabled,
            });
        }
        let lines = (buttons.len() as f32 / per_row).ceil().max(1.0);
        cursor += lines * (row + gap) + gap;
        // Create and inspect.
        let half = (w - gap) / 2.0;
        out.push(Widget {
            act: Act::AddKey,
            rect: [x, cursor, half, row],
            label: if self.capturing { "Press a key...".into() } else { "+ Key button".into() },
            active: self.capturing,
            danger: false,
            enabled: true,
        });
        out.push(Widget {
            act: Act::Picker,
            rect: [x + half + gap, cursor, half, row],
            label: "Binding...".into(),
            active: self.picker,
            danger: false,
            enabled: self.selected.is_some(),
        });
        cursor += row + gap;
        let Some(id) = self.selected else {
            return;
        };
        let Some(button) = buttons.iter().find(|b| b.id == id) else {
            return;
        };
        let steps = 3.0;
        let cell = (w - gap * (steps - 1.0)) / steps;
        let props = [
            (Act::Rename, format!("Name: {}", button.label_text()), self.text == Some(TextTarget::Button(id)), true),
            (Act::CycleIcon, format!("Icon: {}", button.icon.label()), false, true),
            (Act::CycleBehavior, format!("Mode: {}", behavior_label(button.behavior)), false, true),
            (Act::Size(false), "Size -".into(), false, true),
            (Act::Size(true), "Size +".into(), false, true),
            (Act::Opacity(false), "Opacity -".into(), false, true),
            (Act::Opacity(true), "Opacity +".into(), false, true),
            (Act::ToggleEnabled, if button.enabled { "Enabled".into() } else { "Disabled".into() }, button.enabled, true),
            (Act::CycleVisibility, format!("Shown: {}", button.visibility.label()), false, true),
            (Act::Front, "Bring front".into(), false, true),
            (Act::DeleteButton, if self.confirm_delete { "Confirm delete".into() } else { "Delete".into() }, self.confirm_delete, button.id.is_custom()),
            (Act::Dismiss, format!("Opacity {:.0}%", button.opacity * 100.0), false, false),
        ];
        for (i, (act, label, active, enabled)) in props.into_iter().enumerate() {
            let col = i as f32 % steps;
            let line = (i as f32 / steps).floor();
            out.push(Widget {
                act,
                rect: [x + col * (cell + gap), cursor + line * (row + gap), cell, row],
                label,
                active,
                danger: matches!(act, Act::DeleteButton),
                enabled,
            });
        }
    }

    fn layout_widgets(
        &self,
        ed: &Editor,
        out: &mut Vec<Widget>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        gap: f32,
        row: f32,
    ) {
        let _ = h;
        let per_row = 3.0;
        let chip_w = (w - gap * (per_row - 1.0)) / per_row;
        for (i, layout) in ed.staged.items.iter().enumerate() {
            let col = i as f32 % per_row;
            let line = (i as f32 / per_row).floor();
            let default = i == ed.staged.default;
            out.push(Widget {
                act: Act::Layout(i),
                rect: [x + col * (chip_w + gap), y + line * (row + gap), chip_w, row],
                label: format!("{}{}", layout.name, if default { " *" } else { "" }),
                active: i == ed.staged.active,
                danger: false,
                enabled: true,
            });
        }
        let lines = (ed.staged.items.len() as f32 / per_row).ceil().max(1.0);
        let mut cursor = y + lines * (row + gap) + gap;
        let actions = [
            (Act::NewLayout, "New".to_string(), false),
            (Act::CopyLayout, "Copy".into(), false),
            (Act::RenameLayout, "Rename".into(), self.text == Some(TextTarget::Layout)),
            (Act::MakeDefault, "Make default".into(), ed.staged.active == ed.staged.default),
            (Act::Mirror, "Mirror".into(), false),
            (Act::Factory, "Factory positions".into(), false),
            (Act::Export, "Export file".into(), false),
            (Act::Import, "Import file".into(), false),
            (
                Act::DeleteLayout,
                if self.confirm_delete { "Confirm delete".into() } else { "Delete".into() },
                false,
            ),
        ];
        let per = 3.0;
        let cell = (w - gap * (per - 1.0)) / per;
        for (i, (act, label, active)) in actions.into_iter().enumerate() {
            let col = i as f32 % per;
            let line = (i as f32 / per).floor();
            out.push(Widget {
                act,
                rect: [x + col * (cell + gap), cursor + line * (row + gap), cell, row],
                label,
                active,
                danger: matches!(act, Act::DeleteLayout),
                enabled: !(matches!(act, Act::DeleteLayout) && ed.staged.items.len() < 2),
            });
        }
        cursor += 3.0 * (row + gap);
        out.push(Widget {
            act: Act::Dismiss,
            rect: [x, cursor, w, row],
            label: "Tap a layout to edit it. Save keeps it; Cancel restores everything.".into(),
            active: false,
            danger: false,
            enabled: false,
        });
    }

    fn position_widgets(
        &self,
        out: &mut Vec<Widget>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        gap: f32,
        row: f32,
    ) {
        let _ = h;
        let half = (w - gap) / 2.0;
        out.push(Widget {
            act: Act::Factory,
            rect: [x, y, half, row],
            label: "Factory positions".into(),
            active: false,
            danger: false,
            enabled: true,
        });
        out.push(Widget {
            act: Act::Mirror,
            rect: [x + half + gap, y, half, row],
            label: "Mirror layout".into(),
            active: false,
            danger: false,
            enabled: true,
        });
        out.push(Widget {
            act: Act::Dismiss,
            rect: [x, y + row + gap, w, row],
            label: "Drag any round button to move it. Save keeps it; Cancel restores everything.".into(),
            active: false,
            danger: false,
            enabled: false,
        });
    }

    fn picker_widgets(
        &self,
        out: &mut Vec<Widget>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        gap: f32,
        row: f32,
    ) {
        let choices = Self::choices(self.picker_raw);
        let start = self.picker_page * PICKER_ROWS;
        let list = (h - 2.0 * (row + gap)).max(row);
        let per_row = 2.0;
        let cell = (w - gap) / per_row;
        let line_h = (list / PICKER_ROWS as f32).clamp(row * 0.5, row);
        for i in 0..PICKER_ROWS {
            let index = start + i;
            let col = i as f32 % per_row;
            let line = (i as f32 / per_row).floor();
            let enabled = index < choices.len();
            out.push(Widget {
                act: Act::Pick(index),
                rect: [x + col * (cell + gap), y + line * (line_h + gap * 0.5), cell, line_h],
                label: choices.get(index).map(|c| c.1.clone()).unwrap_or_default(),
                active: false,
                danger: false,
                enabled,
            });
        }
        let bottom = y + 4.0 * (line_h + gap * 0.5) + gap;
        let cell3 = (w - 2.0 * gap) / 3.0;
        out.push(Widget {
            act: Act::PickerPage(false),
            rect: [x, bottom, cell3 * 0.5, row],
            label: "<".into(),
            active: false,
            danger: false,
            enabled: self.picker_page > 0,
        });
        out.push(Widget {
            act: Act::PickerPage(true),
            rect: [x + cell3 * 0.5 + gap, bottom, cell3 * 0.5, row],
            label: ">".into(),
            active: false,
            danger: false,
            enabled: self.picker_page + 1 < Self::page_count(self.picker_raw),
        });
        let cell2 = (w - 2.0 * gap) / 2.0;
        out.push(Widget {
            act: Act::PickerRaw,
            rect: [x + cell3 + gap, bottom, cell2, row],
            label: if self.picker_raw { "Physical keys".into() } else { "Game actions".into() },
            active: self.picker_raw,
            danger: false,
            enabled: true,
        });
        out.push(Widget {
            act: Act::ClosePicker,
            rect: [x + w - cell2, bottom, cell2, row],
            label: "Close".into(),
            active: false,
            danger: true,
            enabled: true,
        });
    }

    /// The topmost widget under a surface point, in fractions of the surface.
    pub fn hit(&self, ed: &Editor, w: f32, h: f32, point: [f32; 2]) -> Option<Act> {
        if !point.into_iter().all(f32::is_finite) {
            return None;
        }
        let pixels = [point[0] * w, point[1] * h];
        self.widgets(ed, w, h)
            .iter()
            .rev()
            .find(|w| w.enabled && contains(w.rect, pixels))
            .map(|w| w.act)
    }

    /// Draw the whole overlay. Widgets with actions keep their hit rectangles.
    pub fn draw(&self, b: &mut HudBatch, a: &HudAtlas, ed: &Editor) {
        let (w, h) = (b.width, b.height);
        let [x, y, aw, ah] = Self::area(w, h);
        b.fill(a, 0.0, 0.0, w, h, rgba(4, 2, 10, 150));
        panel(b, a, [x - 3.0, y - 3.0, aw + 6.0, ah + 6.0], rgba(12, 8, 20, 232));
        for widget in self.widgets(ed, w, h) {
            chip(b, a, widget.rect, &widget.label, widget.active, widget.danger, widget.enabled);
        }
        if self.capturing {
            let rect = [w * 0.5 - h * 0.28, h * 0.5 - h * 0.05, h * 0.56, h * 0.1];
            panel(b, a, rect, rgba(60, 18, 110, 240));
            label(b, a, "Press any key to bind it", rect[0], rect[1], rect[2], rect[3], rgba(245, 235, 255, 255));
        }
        if let Some(message) = &self.message {
            let rect = [x, y + ah - Self::row_height(h), aw, Self::row_height(h)];
            panel(b, a, rect, rgba(30, 12, 48, 240));
            label(b, a, message, rect[0], rect[1], rect[2], rect[3], rgba(226, 190, 255, 255));
        }
    }

    /// The neutral hint drawn under the widgets when no message is showing.
    pub fn hint(&self) -> &'static str {
        if self.picker {
            "Choose the binding this button sends to the engine."
        } else if self.capturing {
            "Press a physical key. It becomes this button's binding."
        } else {
            match self.tab {
                Tab::Buttons => "Select a button, then edit its binding, size, opacity and mode.",
                Tab::Layouts => "Named layouts: create, copy, rename, delete and choose the default.",
                Tab::Position => "Drag buttons to place them, or use the factory arrangement.",
            }
        }
    }
}

fn behavior_label(behavior: Behavior) -> &'static str {
    match behavior {
        Behavior::Hold => "Hold",
        Behavior::Toggle => "Toggle",
        Behavior::Single => "Single",
    }
}

fn contains([x, y, w, h]: [f32; 4], p: [f32; 2]) -> bool {
    p[0] >= x && p[1] >= y && p[0] < x + w && p[1] < y + h
}

/// Dark glass card with a soft neon edge, drawn from strips so the corners are
/// actually round on any surface size.
pub fn panel(b: &mut HudBatch, a: &HudAtlas, rect: [f32; 4], color: u32) {
    let [x, y, w, h] = rect;
    let radius = (h * 0.12).min(w * 0.12).max(2.0);
    rounded(b, a, [x - 1.5, y - 1.5, w + 3.0, h + 3.0], radius, rgba(150, 60, 230, 120));
    rounded(b, a, rect, radius, color);
}

fn rounded(b: &mut HudBatch, a: &HudAtlas, [x, y, w, h]: [f32; 4], radius: f32, color: u32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.clamp(0.0, w.min(h) * 0.5);
    b.fill(a, x, y + r, w, h - 2.0 * r, color);
    let steps = (r / 1.5).ceil().max(1.0);
    let step = r / steps;
    for i in 0..steps as usize {
        let yy = i as f32 * step;
        let middle = (r - yy - step * 0.5).max(0.0);
        let inset = r - (r * r - middle * middle).max(0.0).sqrt();
        b.fill(a, x + inset, y + yy, w - 2.0 * inset, step, color);
        b.fill(a, x + inset, y + h - yy - step, w - 2.0 * inset, step, color);
    }
}

fn chip(
    b: &mut HudBatch,
    a: &HudAtlas,
    rect: [f32; 4],
    text: &str,
    active: bool,
    danger: bool,
    enabled: bool,
) {
    let (fill, edge, ink) = if danger && enabled {
        (rgba(96, 18, 40, 232), rgba(255, 110, 150, 190), rgba(255, 225, 235, 255))
    } else if active {
        (rgba(86, 26, 148, 236), rgba(210, 140, 255, 230), rgba(250, 242, 255, 255))
    } else if !enabled {
        (rgba(16, 12, 24, 190), rgba(90, 60, 120, 90), rgba(150, 135, 165, 200))
    } else {
        (rgba(22, 15, 34, 224), rgba(160, 90, 220, 150), rgba(238, 232, 250, 255))
    };
    let [x, y, w, h] = rect;
    rounded(b, a, [x - 1.0, y - 1.0, w + 2.0, h + 2.0], (h * 0.32).max(2.0), edge);
    rounded(b, a, rect, (h * 0.32).max(2.0), fill);
    if text.is_empty() {
        return;
    }
    let width = HudBatch::text_width(a, text);
    let start = b.vertices.len();
    b.text(
        a,
        text,
        x + (w - width) / 2.0,
        y + (h - 8.0) / 2.0,
        ink,
        false,
    );
    let scale = ((w - 5.0).max(1.0) / width.max(1.0)).min(1.0);
    b.scale_since(start, x + w / 2.0, y + h / 2.0, scale, scale);
}

fn label(b: &mut HudBatch, a: &HudAtlas, text: &str, x: f32, y: f32, w: f32, h: f32, ink: u32) {
    let width = HudBatch::text_width(a, text);
    let start = b.vertices.len();
    b.text(a, text, x + (w - width) / 2.0, y + (h - 8.0) / 2.0, ink, false);
    let scale = ((w - 4.0).max(1.0) / width.max(1.0)).min(1.0);
    b.scale_since(start, x + w / 2.0, y + h / 2.0, scale, scale);
}

/// Icons a custom button may show; kept here so the editor can cycle them.
pub fn icon_label(icon: Icon) -> &'static str {
    icon.label()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::craftix_controls::editor::Layouts;

    fn editor() -> Editor {
        Editor::new(Layouts::default()).unwrap()
    }

    #[test]
    fn every_visible_widget_stays_inside_the_surface() {
        for (w, h) in [(1920.0, 1080.0), (2400.0, 1080.0), (800.0, 480.0), (640.0, 360.0)] {
            let ed = editor();
            for tab in Tab::ALL {
                let ui = EditorUi { tab, ..EditorUi::default() };
                for widget in ui.widgets(&ed, w, h) {
                    let [x, y, cw, ch] = widget.rect;
                    assert!(x >= 0.0 && y >= 0.0, "{:?} off-surface at {w}x{h}", widget.act);
                    assert!(x + cw <= w + 0.5 && y + ch <= h + 0.5, "{:?} spills at {w}x{h}", widget.act);
                    assert!(cw > 0.0 && ch > 0.0);
                }
            }
        }
    }

    #[test]
    fn tabs_save_and_cancel_are_clickable_and_distinct() {
        let ed = editor();
        let ui = EditorUi::default();
        let widgets = ui.widgets(&ed, 1920.0, 1080.0);
        let mut rects = Vec::new();
        for act in [Act::Tab(Tab::Layouts), Act::Tab(Tab::Position), Act::Save, Act::Cancel] {
            let widget = widgets.iter().find(|w| w.act == act).unwrap();
            let p = [
                (widget.rect[0] + widget.rect[2] / 2.0) / 1920.0,
                (widget.rect[1] + widget.rect[3] / 2.0) / 1080.0,
            ];
            assert_eq!(ui.hit(&ed, 1920.0, 1080.0, p), Some(act));
            rects.push(widget.rect);
        }
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(a[0] + a[2] <= b[0] || b[0] + b[2] <= a[0] || a[1] + a[3] <= b[1] || b[1] + b[3] <= a[1]);
            }
        }
    }

    #[test]
    fn selecting_a_button_reveals_its_properties_and_disabled_widgets_are_not_hit() {
        let ed = editor();
        let mut ui = EditorUi::default();
        let acts = ui.widgets(&ed, 1920.0, 1080.0).into_iter().map(|w| w.act).collect::<Vec<_>>();
        assert!(!acts.contains(&Act::CycleBehavior));
        ui.selected = Some(Id::Attack);
        let widgets = ui.widgets(&ed, 1920.0, 1080.0);
        let behavior = widgets.iter().find(|w| w.act == Act::CycleBehavior).unwrap();
        assert!(behavior.enabled);
        ui.picker = true;
        let close = ui
            .widgets(&ed, 1920.0, 1080.0)
            .into_iter()
            .find(|w| w.act == Act::ClosePicker)
            .unwrap();
        let p = [(close.rect[0] + 2.0) / 1920.0, (close.rect[1] + 2.0) / 1080.0];
        assert_eq!(ui.hit(&ed, 1920.0, 1080.0, p), Some(Act::ClosePicker));
    }

    #[test]
    fn picker_pages_cover_every_real_binding_and_never_exceed_the_engine_list() {
        for raw in [false, true] {
            let choices = EditorUi::choices(raw);
            assert!(!choices.is_empty());
            assert!(choices.iter().all(|(token, _)| crate::craftix_controls::layout::valid_token(token) || token.starts_with("action:")));
            let pages = EditorUi::page_count(raw);
            assert!(pages >= 1 && pages * PICKER_ROWS >= choices.len());
            assert!((pages - 1) * PICKER_ROWS < choices.len());
        }
        let raw = EditorUi::choices(true);
        assert!(raw.iter().any(|(token, _)| token == "key.keyboard.m"));
    }

    #[test]
    fn painting_the_whole_editor_generates_bounded_finite_geometry() {
        let ed = editor();
        for tab in Tab::ALL {
            let mut ui = EditorUi { tab, selected: Some(Id::Jump), ..EditorUi::default() };
            let mut b = HudBatch::new([1920, 1080]);
            let a = HudAtlas::boot();
            ui.draw(&mut b, &a, &ed);
            assert!(!b.vertices.is_empty());
            assert!(b.vertices.len() < 200_000, "{tab:?} grew to {}", b.vertices.len());
            assert!(b.vertices.iter().all(|v| v.pos[0].is_finite() && v.pos[1].is_finite()));
            ui.message = Some("Saved".into());
            ui.capturing = true;
            ui.draw(&mut b, &a, &ed);
        }
    }
}
