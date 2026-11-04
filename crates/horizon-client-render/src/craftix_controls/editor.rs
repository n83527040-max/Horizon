//! Transactional, named layouts for the replacement editor.
use super::layout::{Action, Behavior, Button, Circle, Icon, Id, Layout};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::{self, Write}, path::Path};
const MAX_BYTES: usize = 128 * 1024;
const MAX_LAYOUTS: usize = 16;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layouts { pub schema: u8, pub active: usize, pub items: Vec<Layout> }
impl Default for Layouts {
    fn default() -> Self { Self { schema: 1, active: 0, items: vec![Layout::default()] } }
}
impl Layouts {
    pub fn valid(&self) -> bool {
        let mut names = BTreeSet::new();
        self.schema == 1 && !self.items.is_empty() && self.items.len() <= MAX_LAYOUTS
            && self.active < self.items.len()
            && self.items.iter().all(|l| l.valid() && names.insert(l.name.trim().to_lowercase()))
    }
    pub fn from_json(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_BYTES { return Err(invalid("layout file too large")); }
        let data: Self = serde_json::from_slice(bytes).map_err(|_| invalid("invalid layout JSON"))?;
        if !data.valid() { return Err(invalid("invalid layout data")); }
        Ok(data)
    }
    pub fn to_json(&self) -> io::Result<Vec<u8>> {
        if !self.valid() { return Err(invalid("invalid layout data")); }
        let data = serde_json::to_vec_pretty(self)?;
        if data.len() > MAX_BYTES { return Err(invalid("layout file too large")); }
        Ok(data)
    }
    /// Write a fresh temporary sibling and replace only after successful sync.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = self.to_json()?;
        let temp = path.with_extension("craftix-tmp");
        let mut f = std::fs::OpenOptions::new().create_new(true).write(true).open(&temp)?;
        let result = (|| { f.write_all(&bytes)?; f.sync_all()?; drop(f); std::fs::rename(&temp,path) })();
        if result.is_err() { let _ = std::fs::remove_file(&temp); }
        result
    }
}
fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidData,message) }
/// Edits are isolated until commit; cancel returns the untouched original.
pub struct Editor { original: Layouts, pub staged: Layouts, drag: Option<(u64, Id, [f32; 2])> }
impl Editor {
    pub fn new(original: Layouts) -> io::Result<Self> {
        if !original.valid() { return Err(invalid("invalid layouts")); }
        Ok(Self { staged: original.clone(), original, drag: None })
    }
    fn available(&self, name: &str, except: Option<usize>) -> bool {
        let name = name.trim();
        !name.is_empty() && name.chars().count() <= 32 && !self.staged.items.iter().enumerate()
            .any(|(i, l)| Some(i) != except && l.name.trim().to_lowercase() == name.to_lowercase())
    }
    pub fn duplicate(&mut self, name: &str) -> io::Result<()> {
        if self.staged.items.len() >= MAX_LAYOUTS || !self.available(name,None) { return Err(invalid("name unavailable")); }
        let mut layout = self.staged.items[self.staged.active].clone(); layout.name = name.trim().into();
        self.staged.items.push(layout); self.staged.active = self.staged.items.len()-1; Ok(())
    }
    pub fn create(&mut self, name: &str) -> io::Result<()> {
        if self.staged.items.len() >= MAX_LAYOUTS || !self.available(name,None) { return Err(invalid("name unavailable")); }
        let layout = Layout { name: name.trim().into(), ..Layout::default() };
        self.staged.items.push(layout); self.staged.active = self.staged.items.len()-1; Ok(())
    }
    pub fn rename(&mut self, name: &str) -> io::Result<()> {
        let i = self.staged.active;
        if !self.available(name,Some(i)) { return Err(invalid("name unavailable")); }
        self.staged.items[i].name = name.trim().into(); Ok(())
    }
    pub fn select(&mut self, index: usize) -> io::Result<()> {
        if index >= self.staged.items.len() { return Err(invalid("invalid index")); }
        self.drag = None; self.staged.active = index; Ok(())
    }
    pub fn delete(&mut self) -> io::Result<()> {
        if self.staged.items.len() == 1 { return Err(invalid("cannot delete the last layout")); }
        self.drag = None;
        self.staged.items.remove(self.staged.active);
        self.staged.active = self.staged.active.min(self.staged.items.len()-1); Ok(())
    }
    /// Add a functioning key binding; an empty or unknown key is rejected.
    pub fn add_key(&mut self, name: &str, key: &str, icon: Icon) -> io::Result<Id> {
        if self.staged.items[self.staged.active].buttons.len() >= 64 || crate::keybinds::key_code(key).is_none() {
            return Err(invalid("invalid or unavailable key"));
        }
        let layout = &mut self.staged.items[self.staged.active];
        let id = (0..u32::MAX).map(Id::Custom).find(|id| !layout.buttons.iter().any(|b| b.id == *id))
            .ok_or_else(|| invalid("no button IDs available"))?;
        let button = Button { id, action: Some(Action::Key(key.into())), circle: Circle { center: [0.7, 0.30], radius: 0.05 },
            opacity: 0.85, behavior: Behavior::Hold, icon, enabled: true, label: name.trim().into() };
        if !button.valid() || button.label.is_empty() { return Err(invalid("invalid button")); }
        layout.buttons.push(button);
        Ok(id)
    }
    /// Validate all requested properties before replacing the active button.
    pub fn edit_button(&mut self, id: Id, replacement: Button) -> io::Result<()> {
        if id != replacement.id || !replacement.valid() { return Err(invalid("invalid button")); }
        let layout = &mut self.staged.items[self.staged.active];
        let button = layout.buttons.iter_mut().find(|b| b.id == id).ok_or_else(|| invalid("button not found"))?;
        *button = replacement; Ok(())
    }
    pub fn delete_button(&mut self, id: Id) -> io::Result<()> {
        if !matches!(id, Id::Custom(_)) { return Err(invalid("only custom buttons may be deleted")); }
        let buttons=&mut self.staged.items[self.staged.active].buttons;
        let index=buttons.iter().position(|b| b.id==id).ok_or_else(|| invalid("button not found"))?;
        buttons.remove(index); Ok(())
    }
    /// Capture one pointer for the preview without moving a button underneath it.
    pub fn begin_drag(&mut self, pointer: u64, point: [f32; 2], aspect: f32) -> bool {
        if self.drag.is_some() || !aspect.is_finite() || aspect <= 0.0 || !point.into_iter().all(f32::is_finite) { return false; }
        let layout=&self.staged.items[self.staged.active];
        if let Some(button)=layout.buttons.iter().rev().find(|b| b.circle.contains(point,aspect)) {
            self.drag=Some((pointer,button.id,[point[0]-button.circle.center[0],point[1]-button.circle.center[1]]));
            true
        } else { false }
    }
    pub fn move_drag(&mut self, pointer: u64, point: [f32; 2], aspect: f32) -> bool {
        if !aspect.is_finite() || aspect <= 0.0 || !point.into_iter().all(f32::is_finite) { return false; }
        let Some((owner,id,offset))=self.drag else { return false; };
        if owner != pointer { return false; }
        let Some(button)=self.staged.items[self.staged.active].buttons.iter_mut().find(|b| b.id==id) else { self.drag=None; return false; };
        let x_margin=(button.circle.radius/aspect).min(0.5);
        let y_margin=button.circle.radius.min(0.5);
        button.circle.center=[(point[0]-offset[0]).clamp(x_margin,1.0-x_margin),
            (point[1]-offset[1]).clamp(y_margin,1.0-y_margin)];
        true
    }
    pub fn end_drag(&mut self, pointer: u64) {
        if self.drag.is_some_and(|(owner,_,_)| owner==pointer) { self.drag=None; }
    }
    /// Call when the viewport or context changes during editing.
    pub fn cancel_drag(&mut self) { self.drag=None; }
    pub fn reset(&mut self) {
        self.drag = None;
        let name = self.staged.items[self.staged.active].name.clone();
        self.staged.items[self.staged.active] = Layout { name, ..Layout::default() };
    }
    pub fn commit(self) -> io::Result<Layouts> {
        if !self.staged.valid() { return Err(invalid("invalid staged layouts")); }
        Ok(self.staged)
    }
    pub fn cancel(self) -> Layouts { self.original }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_names_copy_reset_delete_and_cancel_are_transactional() {
        let original=Layouts::default(); let mut e=Editor::new(original.clone()).unwrap();
        e.duplicate("Gaming").unwrap(); assert_eq!(e.staged.active,1);
        e.rename("Creative").unwrap(); assert!(e.create("creative").is_err());
        e.staged.items[1].buttons[0].circle.radius=0.2; e.reset();
        assert_eq!(e.staged.items[1].buttons[0].circle.radius,0.15);
        e.delete().unwrap(); assert_eq!(e.cancel(),original);
    }
    #[test]
    fn preview_drag_is_owned_clamped_and_cancelled_transactionally() {
        let old=Layouts::default(); let mut editor=Editor::new(old.clone()).unwrap();
        let id=editor.add_key("Map","key.keyboard.m",Icon::Map).unwrap();
        let p=editor.staged.items[0].buttons.iter().find(|b| b.id==id).unwrap().circle.center;
        assert!(editor.begin_drag(11,p,16.0/9.0));
        assert!(!editor.move_drag(12,[0.8,0.5],16.0/9.0));
        assert!(editor.move_drag(11,[2.0,2.0],16.0/9.0));
        let c=editor.staged.items[0].buttons.iter().find(|b| b.id==id).unwrap().circle;
        assert!(c.center[0]+c.radius/(16.0/9.0) <= 1.0 && c.center[1]+c.radius<=1.0);
        editor.cancel_drag(); assert!(!editor.move_drag(11,[0.4,0.4],16.0/9.0));
        assert_eq!(editor.cancel(),old);
    }
    #[test]
    fn custom_map_button_is_editable_persistent_and_not_a_fake_binding() {
        let mut editor=Editor::new(Layouts::default()).unwrap();
        assert!(editor.add_key("Broken", "key.keyboard.not-real", Icon::Text).is_err());
        let id=editor.add_key("Map", "key.keyboard.m", Icon::Map).unwrap();
        let mut button=editor.staged.items[0].buttons.iter().find(|b| b.id==id).unwrap().clone();
        button.circle.center=[0.77,0.31]; button.circle.radius=0.08; button.opacity=0.5; button.behavior=Behavior::Toggle;
        editor.edit_button(id,button.clone()).unwrap();
        let settings=editor.commit().unwrap(); let restored=Layouts::from_json(&settings.to_json().unwrap()).unwrap();
        assert_eq!(restored.items[0].buttons.iter().find(|b| b.id==id),Some(&button));
        let mut editor=Editor::new(restored).unwrap(); editor.delete_button(id).unwrap();
        assert!(!editor.staged.items[0].buttons.iter().any(|b| b.id==id));
    }
    #[test]
    fn saved_layouts_round_trip_and_invalid_import_is_rejected() {
        let mut e=Editor::new(Layouts::default()).unwrap(); e.create("Two").unwrap();
        let settings=e.commit().unwrap(); let bytes=settings.to_json().unwrap();
        assert_eq!(Layouts::from_json(&bytes).unwrap(),settings);
        assert!(Layouts::from_json(br#"{"schema":1,"active":99,"items":[]}"#).is_err());
        assert!(Layouts::from_json(&vec![b' ';MAX_BYTES+1]).is_err());
        let path=std::env::temp_dir().join(format!("craftix-layouts-test-{}.json",std::process::id()));
        let _=std::fs::remove_file(&path); let _=std::fs::remove_file(path.with_extension("craftix-tmp"));
        settings.save(&path).unwrap();
        assert_eq!(Layouts::from_json(&std::fs::read(&path).unwrap()).unwrap(),settings);
        std::fs::remove_file(path).unwrap();
    }
}
