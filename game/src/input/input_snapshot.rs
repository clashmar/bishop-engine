// game/src/input/input_snapshot.rs
use crate::input::input_table::*;
use std::collections::HashMap;
use macroquad::prelude::*;

#[derive(Clone, Default)]
pub struct InputSnapshot {
    pub down: HashMap<&'static str, bool>,
    pub pressed: HashMap<&'static str, bool>,
    pub released: HashMap<&'static str, bool>,
}
impl InputSnapshot {
    /// Fill `snapshot.keys` with the raw key names that are currently down.
    pub fn capture_input_state(&mut self) {
        // Clear previous frame data
        self.down.clear();
        self.pressed.clear();
        self.released.clear();

        // Keyboard
        for &(name, code) in KEY_TABLE {
            self.down.insert(name, is_key_down(code));
            self.pressed.insert(name, is_key_pressed(code));
            self.released.insert(name, is_key_released(code));
        }

        // Mouse
        for &(name, button) in MOUSE_TABLE {
            self.down.insert(name, is_mouse_button_down(button));
            self.pressed.insert(name, is_mouse_button_pressed(button));
            self.released.insert(name, is_mouse_button_released(button));
        }
    }
}