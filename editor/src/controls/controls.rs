// editor/src/controls/controls.rs
use macroquad::prelude::*;

pub struct Controls;

impl Controls {
    pub fn save() -> bool {
        is_key_pressed(KeyCode::S) && 
        (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
    }

    pub fn undo() -> bool {
        is_key_pressed(KeyCode::Z) && 
        (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) &&
        !(is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
    }

    pub fn redo() -> bool {
        is_key_pressed(KeyCode::Z) && 
        is_key_down(KeyCode::LeftControl) && 
        (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
    }

    pub fn delete() -> bool {
        is_key_pressed(KeyCode::Backspace)
    }

    pub fn copy() -> bool {
        is_key_down(KeyCode::LeftControl) &&
        is_key_pressed(KeyCode::C)
    }

    pub fn paste() -> bool {
        is_key_down(KeyCode::LeftControl) &&
        is_key_pressed(KeyCode::V)
    }

    pub fn v() -> bool {
        is_key_pressed(KeyCode::V) && modifier_not_pressed()
    }
}

fn modifier_not_pressed() -> bool {
    !is_key_down(KeyCode::LeftControl)
    && !is_key_down(KeyCode::RightControl)
    && !is_key_down(KeyCode::LeftShift)
    && !is_key_down(KeyCode::RightShift)
    && !is_key_down(KeyCode::LeftAlt)
    && !is_key_down(KeyCode::RightAlt)
}