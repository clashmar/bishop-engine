// engine_core/src/input.rs
use macroquad::prelude::*;

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

pub fn get_horizontal_input() -> f32 {
    let mut dir_x = 0.0;

    if is_key_down(KeyCode::Right) { dir_x += 1.0; }
    if is_key_down(KeyCode::Left)  { dir_x -= 1.0; }

    dir_x
}