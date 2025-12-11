// engine_core/src/input/input_snapshot.rs
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

pub fn get_omni_input() -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::Right) { dir.x += 1.0; }
    if is_key_down(KeyCode::Left)  { dir.x -= 1.0; }
    if is_key_down(KeyCode::Down)  { dir.y += 1.0; }
    if is_key_down(KeyCode::Up)    { dir.y -= 1.0; }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        dir
    }
}

pub fn get_omni_input_pressed() -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_pressed(KeyCode::Right) { dir.x += 1.0; }
    if is_key_pressed(KeyCode::Left)  { dir.x -= 1.0; }
    if is_key_pressed(KeyCode::Down)  { dir.y += 1.0; }
    if is_key_pressed(KeyCode::Up)    { dir.y -= 1.0; }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        dir
    }
}

pub fn get_horizontal_input() -> f32 {
    let mut dir_x = 0.0;

    if is_key_down(KeyCode::Right) { dir_x += 1.0; }
    if is_key_down(KeyCode::Left)  { dir_x -= 1.0; }

    dir_x
}

pub fn jump() -> bool {
    if is_key_pressed(KeyCode::Space) {
        return true
    }
    false
}