// engine_core/src/controls/controls.rs
use macroquad::prelude::*;

pub struct Controls;

impl Controls {
    pub fn save() -> bool {
        is_key_pressed(KeyCode::S) && 
        (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
        && !(is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
    }

    pub fn save_as() -> bool {
        is_key_pressed(KeyCode::S) && 
        (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
        && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
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

    pub fn escape() -> bool {
        is_key_pressed(KeyCode::Escape) && modifier_not_pressed()
    }

    pub fn enter() -> bool {
        is_key_pressed(KeyCode::Enter) && modifier_not_pressed()
    }

    pub fn c() -> bool {
        is_key_pressed(KeyCode::C) && modifier_not_pressed()
    }

    pub fn d() -> bool {
        is_key_pressed(KeyCode::D) && modifier_not_pressed()
    }

    pub fn e() -> bool {
        is_key_pressed(KeyCode::E) && modifier_not_pressed()
    }

    pub fn g() -> bool {
        is_key_pressed(KeyCode::G) && modifier_not_pressed()
    }

    pub fn h() -> bool {
        is_key_pressed(KeyCode::H) && modifier_not_pressed()
    }

    pub fn m() -> bool {
        is_key_pressed(KeyCode::M) && modifier_not_pressed()
    }

    pub fn r() -> bool {
        is_key_pressed(KeyCode::R) && modifier_not_pressed()
    }

    pub fn s() -> bool {
        is_key_pressed(KeyCode::S) && modifier_not_pressed()
    }

    pub fn t() -> bool {
        is_key_pressed(KeyCode::T) && modifier_not_pressed()
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