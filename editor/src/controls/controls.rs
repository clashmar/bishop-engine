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
        (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
    }

    pub fn redo() -> bool {
        is_key_pressed(KeyCode::Z) && 
        is_key_down(KeyCode::LeftControl) && 
        (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
    }

    pub fn delete() -> bool {
        is_key_pressed(KeyCode::Backspace)
    }
}