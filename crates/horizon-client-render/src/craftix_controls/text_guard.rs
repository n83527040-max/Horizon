//! Keep editor IME key presses out of the gameplay/mod shortcut path while
//! allowing releases for hardware keys already held before text editing.
use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct TextKeyGuard { suppressed: HashSet<KeyCode> }
impl TextKeyGuard {
    pub fn suppress(&mut self, code: Option<KeyCode>, down: bool, editing: bool, was_hardware_held: bool) -> bool {
        let Some(key)=code else { return editing; };
        if !down && self.suppressed.remove(&key) { return true; }
        if down && editing {
            if !was_hardware_held { self.suppressed.insert(key); }
            return true;
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_text_never_enters_mod_path_but_preexisting_hardware_releases() {
        let mut g=TextKeyGuard::default(); let k=Some(KeyCode::KeyK);
        assert!(g.suppress(k,true,true,false));
        assert!(g.suppress(k,false,false,false),"release remains suppressed after editor closes");
        assert!(!g.suppress(k,true,false,false));
        assert!(g.suppress(k,true,true,true),"held key repeat is suppressed during text input");
        assert!(!g.suppress(k,false,true,true),"existing hardware release still reaches engine");
        assert!(g.suppress(None,true,true,false));
        assert!(!g.suppress(None,true,false,false));
    }
}
