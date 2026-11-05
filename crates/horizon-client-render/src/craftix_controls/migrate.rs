//! Explicit, non-destructive preview conversion from 0.8.0 touch settings.
//!
//! This never reads or writes a player's files. The host must preserve the old
//! file and ask the player before saving a migration, and surface warnings.
use super::{editor::Layouts, layout::{Action, Behavior, Button, Circle, Icon, Id, Layout, Visibility}};
use crate::touch::{Control, CustomIcon, CustomVisibility, Mode, Settings};

pub struct Migration { pub layouts: Layouts, pub warnings: Vec<String> }

pub fn preview(source: &Settings) -> Migration {
    let mut settings = source.clone();
    let original_count = settings.custom_buttons.len();
    settings.sanitize();
    let mut warnings = Vec::new();
    if settings.custom_buttons.len() != original_count {
        warnings.push("Some invalid or excess custom buttons could not be converted; the original file is unchanged.".into());
    }
    if settings.mode == Mode::Dpad {
        warnings.push("The old D-pad is converted to the new joystick; D-pad behavior is not implemented.".into());
    }
    if settings.sensitivity < 0.25 {
        warnings.push("Camera sensitivity was clamped to the new minimum; independent axes need manual adjustment.".into());
    } else if settings.sensitivity_horizontal != settings.sensitivity_vertical {
        warnings.push("Independent camera axes require manual adjustment; the new layout has one sensitivity value.".into());
    }
    if settings.crosshair_gestures || settings.contextual_interact || settings.gyroscope {
        warnings.push("Old world-tap, contextual-interact and gyro behaviors are not migrated by this preview.".into());
    }
    warnings.push("External numbered layout slots are separate files and are not imported by this preview.".into());
    let active = settings.mode.index();
    let mut items = Vec::new();
    for (mode, name) in [(Mode::Crosshair, "Legacy crosshair"), (Mode::Tap, "Legacy tap"), (Mode::Dpad, "Legacy D-pad")] {
        settings.mode = mode;
        let mut layout = Layout { name: name.into(), camera_sensitivity: settings.sensitivity.clamp(0.25,3.0), ..Layout::default() };
        for (id, control) in [
            (Id::Joystick,Control::Move), (Id::Sprint,Control::Sprint), (Id::Jump,Control::Jump),
            (Id::Sneak,Control::Sneak), (Id::Attack,Control::Attack), (Id::Use,Control::Use),
            (Id::Chat,Control::Chat), (Id::Pause,Control::Pause), (Id::Settings,Control::Settings),
            (Id::Inventory,Control::Inventory), (Id::Perspective,Control::Perspective),
        ] {
            let old = settings.placement(control);
            let button = layout.buttons.iter_mut().find(|button| button.id == id).expect("factory control");
            button.circle = circle(old, settings.button_scale);
            button.opacity = old.opacity;
            button.enabled = settings.visible(control);
            if id == Id::Sprint && settings.sprint_toggle { button.behavior = Behavior::Toggle; }
        }
        for (index, old) in settings.custom_buttons.iter().enumerate() {
            let mut place = old.layouts[mode.index()];
            if settings.left_handed { place.center[0] = 1.0 - place.center[0]; }
            let action = if let Some(name) = old.key.strip_prefix("action:") {
                Some(Action::Binding(name.into()))
            } else if crate::keybinds::key_code(&old.key).is_some() {
                Some(Action::Key(old.key.clone()))
            } else { None };
            let enabled = old.enabled && action.is_some();
            if action.is_none() && mode == Mode::Crosshair {
                warnings.push(format!("Custom button {} has no supported key and remains disabled.", index + 1));
            }
            let label = if old.name.is_empty() { old.key.chars().take(32).collect() } else { old.name.clone() };
            layout.buttons.push(Button {
                id: Id::Custom(index as u32), action, circle: circle(place,settings.button_scale),
                opacity: place.opacity, enabled, label,
                behavior: if old.single { Behavior::Single } else if old.toggle { Behavior::Toggle } else { Behavior::Hold },
                icon: match old.icon {
                    CustomIcon::Text => Icon::Text, CustomIcon::Attack => Icon::Sword, CustomIcon::Use => Icon::Use,
                    CustomIcon::Jump => Icon::Jump, CustomIcon::Sneak => Icon::Sneak, CustomIcon::Sprint => Icon::Sprint,
                },
                visibility: match old.visibility {
                    CustomVisibility::Always => Visibility::Always, CustomVisibility::Gameplay => Visibility::Gameplay,
                    CustomVisibility::Inventory => Visibility::Inventory,
                },
            });
        }
        items.push(layout);
    }
    if settings.offhand_button || settings.shield_button {
        warnings.push("Legacy dedicated offhand/shield controls need manual recreation in the new layout.".into());
    }
    Migration { layouts: Layouts { schema: 1, active, items }, warnings }
}
fn circle(place: crate::touch::layout::Placement, scale: f32) -> Circle {
    Circle { center: place.center, radius: (place.size * scale * 0.5).clamp(0.02,0.24) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::touch::{CustomButton, CustomVisibility};
    #[test]
    fn preview_preserves_old_file_data_and_custom_meaning() {
        let mut old=Settings::default(); old.mode=Mode::Tap; old.left_handed=true;
        old.sprint_toggle=true;
        let mut custom=CustomButton::new("key.keyboard.m".into(),0);
        custom.name="Map".into(); custom.toggle=true; custom.visibility=CustomVisibility::Inventory;
        old.custom_buttons.push(custom);
        let before=serde_json::to_vec(&old).unwrap();
        let result=preview(&old);
        assert!(result.layouts.valid()); assert_eq!(result.layouts.active,1);
        assert_eq!(serde_json::to_vec(&old).unwrap(),before);
        let button=result.layouts.items[1].buttons.iter().find(|b| b.id==Id::Custom(0)).unwrap();
        assert_eq!(button.action,Some(Action::Key("key.keyboard.m".into())));
        assert_eq!(button.behavior,Behavior::Toggle); assert_eq!(button.visibility,Visibility::Inventory);
        assert!((button.circle.center[0]-(1.0-old.custom_buttons[0].layouts[1].center[0])).abs()<1e-6);
    }
    #[test]
    fn unbound_placeholder_is_never_a_fake_working_button() {
        let mut old=Settings::default();
        old.custom_buttons.push(CustomButton::new("key.keyboard.unknown".into(),0));
        let result=preview(&old); assert!(result.layouts.valid());
        let b=result.layouts.items[0].buttons.iter().find(|b| b.id==Id::Custom(0)).unwrap();
        assert!(!b.enabled && b.action.is_none());
        assert!(result.warnings.iter().any(|w| w.contains("no supported key")));
    }
}
