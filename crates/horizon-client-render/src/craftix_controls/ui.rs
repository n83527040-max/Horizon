//! Touch-driven editor for the replacement layout model.
//! Coordinates and hit targets are shared between drawing and interaction.
use super::{editor::{Editor, Layouts}, layout::{Action, Behavior, Icon, Id}, paint};
use crate::hud::{rgba, HudAtlas, HudBatch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page { Buttons, Layouts, Position }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextField { ButtonName, ButtonKey, NewLayout, CopyLayout, LayoutName }
#[derive(Clone, Debug, PartialEq)]
pub enum UiEvent { None, ShowText(String), Import, Export, Save, Cancel }

pub struct EditorUi {
    pub model: Editor,
    pub page: Page,
    selected: usize,
    text: Option<(TextField, String)>,
    pub error: String,
    dragging: Option<u64>,
}
impl EditorUi {
    pub fn new(layouts: Layouts) -> std::io::Result<Self> {
        Ok(Self { model: Editor::new(layouts)?, page: Page::Buttons, selected: 0,
            text: None, error: String::new(), dragging: None })
    }
    fn layout(&self) -> &super::layout::Layout { &self.model.staged.items[self.model.staged.active] }
    fn selected_id(&self) -> Option<Id> { self.layout().buttons.get(self.selected).map(|b| b.id) }
    fn selected_button(&self) -> Option<&super::layout::Button> { self.layout().buttons.get(self.selected) }
    fn text_request(&mut self, field: TextField, initial: String) -> UiEvent {
        self.text = Some((field, initial.clone())); UiEvent::ShowText(initial)
    }
    pub fn cancel_drag(&mut self) { self.dragging=None; self.model.cancel_drag(); }
    pub fn imported(&mut self) { self.cancel_drag(); self.selected=0; }

    pub fn text_active(&self) -> bool { self.text.is_some() }
    pub fn text_insert(&mut self, text: &str) {
        if let Some((_, value)) = &mut self.text { value.extend(text.chars().filter(|c| !c.is_control()).take(128)); }
    }
    pub fn text_backspace(&mut self) { if let Some((_, value)) = &mut self.text { value.pop(); } }
    pub fn finish_text(&mut self) {
        let Some((field, value)) = self.text.take() else { return; };
        let value = value.trim();
        let result = match field {
            TextField::NewLayout => self.model.create(value),
            TextField::CopyLayout => self.model.duplicate(value),
            TextField::LayoutName => self.model.rename(value),
            TextField::ButtonName | TextField::ButtonKey => {
                if let Some(mut button) = self.selected_button().cloned() {
                    if field == TextField::ButtonName { button.label = value.into(); }
                    else {
                        let action = if let Some(name) = value.strip_prefix("action:") {
                            Some(Action::Binding(name.to_owned()))
                        } else { super::layout::parse_key_action(value) };
                        let Some(action) = action else { self.error = "Invalid key or shortcut".into(); return; };
                        button.action = Some(action);
                    }
                    self.model.edit_button(button.id, button)
                } else { return; }
            }
        };
        self.error = result.err().map_or_else(String::new, |e| e.to_string());
        self.selected = self.selected.min(self.layout().buttons.len().saturating_sub(1));
    }
    fn edit_selected(&mut self, change: impl FnOnce(&mut super::layout::Button)) {
        if let Some(mut b) = self.selected_button().cloned() {
            change(&mut b); self.error = self.model.edit_button(b.id,b).err().map_or_else(String::new, |e| e.to_string());
        }
    }
    /// Screen fractions. Position-page drags are owned by one pointer.
    pub fn touch(&mut self, id: u64, point: [f32; 2], aspect: f32, start: bool, end: bool) -> UiEvent {
        if self.text_active() { return UiEvent::None; }
        if self.dragging == Some(id) {
            if end { self.model.end_drag(id); self.dragging = None; }
            else if !start { self.model.move_drag(id,point,aspect); }
            return UiEvent::None;
        }
        if !start { return UiEvent::None; }
        if point[1] < 0.12 {
            self.model.cancel_drag(); self.dragging=None;
            self.page = if point[0] < 0.34 { Page::Buttons } else if point[0] < 0.67 { Page::Layouts } else { Page::Position };
            return UiEvent::None;
        }
        if point[1] > 0.90 {
            return if point[0] < 0.33 { UiEvent::Cancel } else if point[0] < 0.66 {
                self.model.reset(); self.selected=0; UiEvent::None
            } else { UiEvent::Save };
        }
        if self.page == Page::Position {
            if let Some(index) = self.layout().buttons.iter().rposition(|b| b.circle.contains(point,aspect)) { self.selected=index; }
            if self.model.begin_drag(id,point,aspect) { self.dragging=Some(id); }
            else if let Some(selected) = self.selected_id() { self.model.move_button_to(selected,point,aspect); }
            return UiEvent::None;
        }
        let x=point[0]; let y=point[1];
        if self.page == Page::Layouts {
            if y < 0.28 {
                let len=self.model.staged.items.len(); let i=self.model.staged.active;
                let next=if x<0.5 { (i+len-1)%len } else { (i+1)%len };
                let _=self.model.select(next); self.selected=0;
            } else if y < 0.42 {
                return if x<0.5 { self.text_request(TextField::NewLayout,String::new()) }
                    else { self.text_request(TextField::CopyLayout,String::new()) };
            } else if y < 0.56 {
                if x<0.5 { return self.text_request(TextField::LayoutName,self.layout().name.clone()); }
                self.error=self.model.delete().err().map_or_else(String::new, |e| e.to_string()); self.selected=0;
            } else if y < 0.69 {
                let layout=&mut self.model.staged.items[self.model.staged.active];
                layout.joystick_sensitivity=(layout.joystick_sensitivity + if x<0.5 {-0.1} else {0.1}).clamp(0.25,3.0);
            } else if y < 0.82 {
                let layout=&mut self.model.staged.items[self.model.staged.active];
                layout.camera_sensitivity=(layout.camera_sensitivity + if x<0.5 {-0.1} else {0.1}).clamp(0.25,3.0);
            } else if y < 0.90 {
                return if x < 0.5 { UiEvent::Import } else { UiEvent::Export };
            }
            return UiEvent::None;
        }
        if y < 0.28 {
            let len=self.layout().buttons.len();
            if len>0 { self.selected=if x<0.5 { (self.selected+len-1)%len } else { (self.selected+1)%len }; }
        } else if y < 0.40 {
            if x < 0.5 {
                match self.model.add_key("M", "key.keyboard.m", Icon::Text) {
                    Ok(id) => self.selected=self.layout().buttons.iter().position(|b| b.id==id).unwrap_or(0),
                    Err(e) => self.error=e.to_string(),
                }
            } else if let Some(id)=self.selected_id() {
                self.error=self.model.delete_button(id).err().map_or_else(String::new, |e| e.to_string());
                self.selected=self.selected.min(self.layout().buttons.len().saturating_sub(1));
            }
        } else if y < 0.51 {
            if let Some(b)=self.selected_button() { return self.text_request(TextField::ButtonName,b.label.clone()); }
        } else if y < 0.62 {
            if let Some(b)=self.selected_button() {
                if matches!(b.id, Id::Custom(_)) {
                    let initial=match &b.action { Some(Action::Key(key))=>key.clone(), Some(Action::Binding(name))=>format!("action:{name}"), Some(Action::Chord { modifiers, key })=>format!("{}+{key}",modifiers.join("+")), _=>String::new() };
                    return self.text_request(TextField::ButtonKey,initial);
                }
            }
        } else if y < 0.73 {
            if x < 0.33 { self.edit_selected(|b| b.behavior=match b.behavior { Behavior::Hold=>Behavior::Toggle, Behavior::Toggle=>Behavior::Single, Behavior::Single=>Behavior::Hold }); }
            else if x < 0.66 { self.edit_selected(|b| b.icon=match b.icon { Icon::Text=>Icon::Map,Icon::Map=>Icon::Sword,Icon::Sword=>Icon::Use,Icon::Use=>Icon::Jump,Icon::Jump=>Icon::Sneak,Icon::Sneak=>Icon::Sprint,Icon::Sprint=>Icon::Text }); }
            else { self.edit_selected(|b| b.enabled=!b.enabled); }
        } else if y < 0.81 {
            self.edit_selected(|b| b.circle.radius=(0.02+x.clamp(0.0,1.0)*0.22).clamp(0.02,0.24));
        } else if y < 0.90 {
            self.edit_selected(|b| b.opacity=x.clamp(0.0,1.0));
        }
        UiEvent::None
    }
    pub fn draw(&self, b: &mut HudBatch, atlas: &HudAtlas) {
        if self.page == Page::Position { paint::paint(b,atlas,self.layout(),&paint::Animator::default()); }
        else { b.fill(atlas,0.0,0.0,b.width,b.height,rgba(8,6,18,220)); }
        let tabs=["Buttons","Layouts","Position"];
        for (i,label) in tabs.iter().enumerate() { plate(b,atlas,[i as f32/3.0,0.02,0.32,0.09],label,self.page==[Page::Buttons,Page::Layouts,Page::Position][i]); }
        if self.page == Page::Buttons {
            let selected=self.selected_button();
            let label=selected.map_or("No button".into(),|button| format!("{:?} {}",button.id,button.label));
            plate(b,atlas,[0.02,0.16,0.96,0.10],&format!("<  {label}  >"),false);
            plate(b,atlas,[0.02,0.29,0.46,0.09],"+ Add key M",false); plate(b,atlas,[0.52,0.29,0.46,0.09],"Delete custom",false);
            plate(b,atlas,[0.02,0.41,0.96,0.09],&format!("Name: {}",selected.map_or("",|b| b.label.as_str())),false);
            let key=selected.and_then(|b| b.action.as_ref()).map_or("None".into(), |a| format!("{a:?}"));
            plate(b,atlas,[0.02,0.52,0.96,0.09],&format!("Key: {key} (custom only)"),false);
            plate(b,atlas,[0.02,0.63,0.31,0.09],&format!("{:?}",selected.map(|b| b.behavior)),false);
            plate(b,atlas,[0.35,0.63,0.31,0.09],&format!("{:?}",selected.map(|b| b.icon)),false);
            plate(b,atlas,[0.68,0.63,0.30,0.09],if selected.is_some_and(|b| b.enabled) {"Enabled"} else {"Disabled"},false);
            plate(b,atlas,[0.02,0.74,0.96,0.06],&format!("Size: {:.2} — tap to change",selected.map_or(0.0,|b| b.circle.radius)),false);
            plate(b,atlas,[0.02,0.82,0.96,0.06],&format!("Opacity: {:.0}% — tap to change",selected.map_or(0.0,|b| b.opacity*100.0)),false);
        } else if self.page == Page::Layouts {
            plate(b,atlas,[0.02,0.16,0.96,0.10],&format!("<  {}  >",self.layout().name),false);
            plate(b,atlas,[0.02,0.30,0.46,0.10],"Create",false); plate(b,atlas,[0.52,0.30,0.46,0.10],"Duplicate",false);
            plate(b,atlas,[0.02,0.44,0.46,0.10],"Rename",false); plate(b,atlas,[0.52,0.44,0.46,0.10],"Delete",false);
            plate(b,atlas,[0.02,0.58,0.96,0.10],&format!("-  Joystick sensitivity {:.1}  +",self.layout().joystick_sensitivity),false);
            plate(b,atlas,[0.02,0.71,0.96,0.10],&format!("-  Camera sensitivity {:.1}  +",self.layout().camera_sensitivity),false);
            plate(b,atlas,[0.02,0.83,0.46,0.06],"Import transfer JSON",false);
            plate(b,atlas,[0.52,0.83,0.46,0.06],"Export transfer JSON",false);
        }
        if !self.error.is_empty() { plate(b,atlas,[0.02,0.86,0.96,0.035],&self.error,false); }
        plate(b,atlas,[0.01,0.91,0.31,0.08],"Cancel",false);
        plate(b,atlas,[0.34,0.91,0.31,0.08],"Reset",false);
        plate(b,atlas,[0.67,0.91,0.32,0.08],"Save",true);
    }
}
fn plate(b: &mut HudBatch,a: &HudAtlas,rect:[f32;4],label:&str,active:bool) {
    let [x,y,w,h]=[rect[0]*b.width,rect[1]*b.height,rect[2]*b.width,rect[3]*b.height];
    b.fill(a,x,y,w,h,if active {rgba(70,28,103,235)} else {rgba(22,15,36,230)});
    b.fill(a,x,y+h-1.0,w,1.0,rgba(186,91,255,230));
    let start=b.vertices.len(); let text_width=HudBatch::text_width(a,label);
    b.text(a,label,x+(w-text_width.min(w-4.0))/2.0,y+(h-8.0)/2.0,rgba(250,245,255,255),false);
    if text_width>w-4.0 { b.scale_since(start,x+2.0,y+h/2.0,((w-4.0)/text_width).max(0.01),1.0); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_button_edit_save_and_cancel_are_isolated() {
        let original=Layouts::default(); let mut ui=EditorUi::new(original.clone()).unwrap();
        ui.touch(1,[0.2,0.33],16.0/9.0,true,false);
        assert!(matches!(ui.touch(2,[0.5,0.55],16.0/9.0,true,false),UiEvent::ShowText(_)));
        ui.text=Some((TextField::ButtonKey,String::new())); ui.text_insert("m"); ui.finish_text();
        assert_eq!(ui.selected_button().unwrap().action,Some(Action::Key("key.keyboard.m".into())));
        assert_eq!(ui.model.cancel(),original);
    }
    #[test]
    fn layouts_page_exposes_explicit_transfer_actions() {
        let mut ui=EditorUi::new(Layouts::default()).unwrap();
        ui.page=Page::Layouts;
        assert_eq!(ui.touch(1,[0.25,0.85],16.0/9.0,true,false),UiEvent::Import);
        assert_eq!(ui.touch(2,[0.75,0.85],16.0/9.0,true,false),UiEvent::Export);
    }
    #[test]
    fn custom_shortcut_edit_is_validated_without_erasing_the_old_binding() {
        let mut ui=EditorUi::new(Layouts::default()).unwrap();
        ui.touch(1,[0.2,0.33],16.0/9.0,true,false);
        ui.text=Some((TextField::ButtonKey,"Ctrl + M".into())); ui.finish_text();
        let chord=ui.selected_button().unwrap().action.clone();
        assert!(matches!(chord,Some(Action::Chord { .. })));
        ui.text=Some((TextField::ButtonKey,"Ctrl+not-a-key".into())); ui.finish_text();
        assert_eq!(ui.selected_button().unwrap().action,chord); assert!(!ui.error.is_empty());
    }
    #[test]
    fn position_page_can_move_top_bar_button_from_under_tabs() {
        let mut ui=EditorUi::new(Layouts::default()).unwrap();
        ui.selected=ui.layout().buttons.iter().position(|b| b.id==Id::Chat).unwrap();
        ui.page=Page::Position;
        ui.touch(1,[0.5,0.5],16.0/9.0,true,false);
        assert_eq!(ui.selected_button().unwrap().circle.center,[0.5,0.5]);
    }
    #[test]
    fn editor_output_reaches_router_and_real_key_token_bridge() {
        use crate::craftix_controls::{bridge::{Bridge,HostEvent},input::Router};
        let mut ui=EditorUi::new(Layouts::default()).unwrap();
        ui.touch(1,[0.2,0.33],16.0/9.0,true,false);
        let button=ui.selected_button().unwrap().clone();
        let layout=ui.model.commit().unwrap().items.remove(0);
        let mut router=Router::default(); let mut bridge=Bridge::default();
        router.begin(&layout,42,button.circle.center,16.0/9.0,0);
        assert_eq!(bridge.translate(router.drain()),vec![HostEvent::Input("key.keyboard.m".into(),true)]);
        router.end(42);
        assert_eq!(bridge.translate(router.drain()),vec![HostEvent::Input("key.keyboard.m".into(),false)]);
    }
    #[test]
    fn named_layout_create_and_reset_use_the_editor_model() {
        let mut ui=EditorUi::new(Layouts::default()).unwrap();
        ui.touch(1,[0.5,0.05],1.7,true,false);
        assert!(matches!(ui.touch(1,[0.2,0.33],1.7,true,false),UiEvent::ShowText(_)));
        ui.text_insert("Building"); ui.finish_text();
        assert_eq!(ui.layout().name,"Building"); assert!(ui.model.staged.valid());
    }
}
