use crate::*;
use macroquad::prelude as mq;

/// Macroquad backend implementation wrapping global functions.
pub struct MacroquadContext {
    char_buffer: Vec<char>,
}

impl MacroquadContext {
    /// Creates a new macroquad context.
    pub fn new() -> Self {
        Self {
            char_buffer: Vec::new(),
        }
    }

    /// Updates the character buffer. Call once per frame before processing input.
    pub fn update(&mut self) {
        self.char_buffer.clear();
        while let Some(c) = mq::get_char_pressed() {
            self.char_buffer.push(c);
        }
    }
}

impl Default for MacroquadContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for MacroquadContext {
    fn is_key_down(&self, key: KeyCode) -> bool {
        mq::is_key_down(key.into())
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        mq::is_key_pressed(key.into())
    }

    fn is_key_released(&self, key: KeyCode) -> bool {
        mq::is_key_released(key.into())
    }

    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        mq::is_mouse_button_down(button.into())
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        mq::is_mouse_button_pressed(button.into())
    }

    fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        mq::is_mouse_button_released(button.into())
    }

    fn mouse_position(&self) -> (f32, f32) {
        mq::mouse_position()
    }

    fn mouse_wheel(&self) -> (f32, f32) {
        mq::mouse_wheel()
    }

    fn chars_pressed(&self) -> Vec<char> {
        self.char_buffer.clone()
    }
}

impl Draw for MacroquadContext {
    fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        mq::draw_rectangle(x, y, w, h, color.into());
    }

    fn draw_rectangle_lines(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        mq::draw_rectangle_lines(x, y, w, h, thickness, color.into());
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        mq::draw_line(x1, y1, x2, y2, thickness, color.into());
    }

    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        mq::draw_circle(x, y, radius, color.into());
    }

    fn draw_circle_lines(&mut self, x: f32, y: f32, radius: f32, thickness: f32, color: Color) {
        mq::draw_circle_lines(x, y, radius, thickness, color.into());
    }

    fn draw_triangle(&mut self, v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
        mq::draw_triangle(v1.into(), v2.into(), v3.into(), color.into());
    }

    fn clear(&mut self, color: Color) {
        mq::clear_background(color.into());
    }
}

impl Text for MacroquadContext {
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
        let dims = mq::measure_text(text, None, font_size as u16, 1.0);
        mq::draw_text(text, x, y, font_size, color.into());
        TextDimensions {
            width: dims.width,
            height: dims.height,
            offset_y: dims.offset_y,
        }
    }

    fn measure_text(&self, text: &str, font_size: f32) -> TextDimensions {
        let dims = mq::measure_text(text, None, font_size as u16, 1.0);
        TextDimensions {
            width: dims.width,
            height: dims.height,
            offset_y: dims.offset_y,
        }
    }
}

impl DrawTexture for MacroquadContext {
    type Texture = mq::Texture2D;

    fn draw_texture(&mut self, texture: &Self::Texture, x: f32, y: f32, color: Color) {
        mq::draw_texture(texture, x, y, color.into());
    }

    fn draw_texture_ex(
        &mut self,
        texture: &Self::Texture,
        x: f32,
        y: f32,
        color: Color,
        params: DrawTextureParams,
    ) {
        mq::draw_texture_ex(
            texture,
            x,
            y,
            color.into(),
            mq::DrawTextureParams {
                dest_size: params.dest_size.map(|v| v.into()),
                source: params.source.map(|r| r.into()),
                rotation: params.rotation,
                flip_x: params.flip_x,
                flip_y: params.flip_y,
                pivot: params.pivot.map(|v| v.into()),
            },
        );
    }
}

// Type conversions from bishop types to macroquad types

impl From<KeyCode> for mq::KeyCode {
    fn from(key: KeyCode) -> Self {
        match key {
            KeyCode::Space => mq::KeyCode::Space,
            KeyCode::Apostrophe => mq::KeyCode::Apostrophe,
            KeyCode::Comma => mq::KeyCode::Comma,
            KeyCode::Minus => mq::KeyCode::Minus,
            KeyCode::Period => mq::KeyCode::Period,
            KeyCode::Slash => mq::KeyCode::Slash,
            KeyCode::Key0 => mq::KeyCode::Key0,
            KeyCode::Key1 => mq::KeyCode::Key1,
            KeyCode::Key2 => mq::KeyCode::Key2,
            KeyCode::Key3 => mq::KeyCode::Key3,
            KeyCode::Key4 => mq::KeyCode::Key4,
            KeyCode::Key5 => mq::KeyCode::Key5,
            KeyCode::Key6 => mq::KeyCode::Key6,
            KeyCode::Key7 => mq::KeyCode::Key7,
            KeyCode::Key8 => mq::KeyCode::Key8,
            KeyCode::Key9 => mq::KeyCode::Key9,
            KeyCode::Semicolon => mq::KeyCode::Semicolon,
            KeyCode::Equal => mq::KeyCode::Equal,
            KeyCode::A => mq::KeyCode::A,
            KeyCode::B => mq::KeyCode::B,
            KeyCode::C => mq::KeyCode::C,
            KeyCode::D => mq::KeyCode::D,
            KeyCode::E => mq::KeyCode::E,
            KeyCode::F => mq::KeyCode::F,
            KeyCode::G => mq::KeyCode::G,
            KeyCode::H => mq::KeyCode::H,
            KeyCode::I => mq::KeyCode::I,
            KeyCode::J => mq::KeyCode::J,
            KeyCode::K => mq::KeyCode::K,
            KeyCode::L => mq::KeyCode::L,
            KeyCode::M => mq::KeyCode::M,
            KeyCode::N => mq::KeyCode::N,
            KeyCode::O => mq::KeyCode::O,
            KeyCode::P => mq::KeyCode::P,
            KeyCode::Q => mq::KeyCode::Q,
            KeyCode::R => mq::KeyCode::R,
            KeyCode::S => mq::KeyCode::S,
            KeyCode::T => mq::KeyCode::T,
            KeyCode::U => mq::KeyCode::U,
            KeyCode::V => mq::KeyCode::V,
            KeyCode::W => mq::KeyCode::W,
            KeyCode::X => mq::KeyCode::X,
            KeyCode::Y => mq::KeyCode::Y,
            KeyCode::Z => mq::KeyCode::Z,
            KeyCode::LeftBracket => mq::KeyCode::LeftBracket,
            KeyCode::Backslash => mq::KeyCode::Backslash,
            KeyCode::RightBracket => mq::KeyCode::RightBracket,
            KeyCode::GraveAccent => mq::KeyCode::GraveAccent,
            KeyCode::World1 => mq::KeyCode::World1,
            KeyCode::World2 => mq::KeyCode::World2,
            KeyCode::Escape => mq::KeyCode::Escape,
            KeyCode::Enter => mq::KeyCode::Enter,
            KeyCode::Tab => mq::KeyCode::Tab,
            KeyCode::Backspace => mq::KeyCode::Backspace,
            KeyCode::Insert => mq::KeyCode::Insert,
            KeyCode::Delete => mq::KeyCode::Delete,
            KeyCode::Right => mq::KeyCode::Right,
            KeyCode::Left => mq::KeyCode::Left,
            KeyCode::Down => mq::KeyCode::Down,
            KeyCode::Up => mq::KeyCode::Up,
            KeyCode::PageUp => mq::KeyCode::PageUp,
            KeyCode::PageDown => mq::KeyCode::PageDown,
            KeyCode::Home => mq::KeyCode::Home,
            KeyCode::End => mq::KeyCode::End,
            KeyCode::CapsLock => mq::KeyCode::CapsLock,
            KeyCode::ScrollLock => mq::KeyCode::ScrollLock,
            KeyCode::NumLock => mq::KeyCode::NumLock,
            KeyCode::PrintScreen => mq::KeyCode::PrintScreen,
            KeyCode::Pause => mq::KeyCode::Pause,
            KeyCode::F1 => mq::KeyCode::F1,
            KeyCode::F2 => mq::KeyCode::F2,
            KeyCode::F3 => mq::KeyCode::F3,
            KeyCode::F4 => mq::KeyCode::F4,
            KeyCode::F5 => mq::KeyCode::F5,
            KeyCode::F6 => mq::KeyCode::F6,
            KeyCode::F7 => mq::KeyCode::F7,
            KeyCode::F8 => mq::KeyCode::F8,
            KeyCode::F9 => mq::KeyCode::F9,
            KeyCode::F10 => mq::KeyCode::F10,
            KeyCode::F11 => mq::KeyCode::F11,
            KeyCode::F12 => mq::KeyCode::F12,
            KeyCode::F13 => mq::KeyCode::F13,
            KeyCode::F14 => mq::KeyCode::F14,
            KeyCode::F15 => mq::KeyCode::F15,
            KeyCode::F16 => mq::KeyCode::F16,
            KeyCode::F17 => mq::KeyCode::F17,
            KeyCode::F18 => mq::KeyCode::F18,
            KeyCode::F19 => mq::KeyCode::F19,
            KeyCode::F20 => mq::KeyCode::F20,
            KeyCode::F21 => mq::KeyCode::F21,
            KeyCode::F22 => mq::KeyCode::F22,
            KeyCode::F23 => mq::KeyCode::F23,
            KeyCode::F24 => mq::KeyCode::F24,
            KeyCode::F25 => mq::KeyCode::F25,
            KeyCode::Kp0 => mq::KeyCode::Kp0,
            KeyCode::Kp1 => mq::KeyCode::Kp1,
            KeyCode::Kp2 => mq::KeyCode::Kp2,
            KeyCode::Kp3 => mq::KeyCode::Kp3,
            KeyCode::Kp4 => mq::KeyCode::Kp4,
            KeyCode::Kp5 => mq::KeyCode::Kp5,
            KeyCode::Kp6 => mq::KeyCode::Kp6,
            KeyCode::Kp7 => mq::KeyCode::Kp7,
            KeyCode::Kp8 => mq::KeyCode::Kp8,
            KeyCode::Kp9 => mq::KeyCode::Kp9,
            KeyCode::KpDecimal => mq::KeyCode::KpDecimal,
            KeyCode::KpDivide => mq::KeyCode::KpDivide,
            KeyCode::KpMultiply => mq::KeyCode::KpMultiply,
            KeyCode::KpSubtract => mq::KeyCode::KpSubtract,
            KeyCode::KpAdd => mq::KeyCode::KpAdd,
            KeyCode::KpEnter => mq::KeyCode::KpEnter,
            KeyCode::KpEqual => mq::KeyCode::KpEqual,
            KeyCode::LeftShift => mq::KeyCode::LeftShift,
            KeyCode::LeftControl => mq::KeyCode::LeftControl,
            KeyCode::LeftAlt => mq::KeyCode::LeftAlt,
            KeyCode::LeftSuper => mq::KeyCode::LeftSuper,
            KeyCode::RightShift => mq::KeyCode::RightShift,
            KeyCode::RightControl => mq::KeyCode::RightControl,
            KeyCode::RightAlt => mq::KeyCode::RightAlt,
            KeyCode::RightSuper => mq::KeyCode::RightSuper,
            KeyCode::Menu => mq::KeyCode::Menu,
            KeyCode::Back => mq::KeyCode::Back,
            KeyCode::Unknown => mq::KeyCode::Unknown,
        }
    }
}

impl From<MouseButton> for mq::MouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => mq::MouseButton::Left,
            MouseButton::Right => mq::MouseButton::Right,
            MouseButton::Middle => mq::MouseButton::Middle,
        }
    }
}

impl From<Color> for mq::Color {
    fn from(color: Color) -> Self {
        mq::Color::new(color.r, color.g, color.b, color.a)
    }
}

impl From<mq::Color> for Color {
    fn from(color: mq::Color) -> Self {
        Color::new(color.r, color.g, color.b, color.a)
    }
}

impl From<Vec2> for mq::Vec2 {
    fn from(v: Vec2) -> Self {
        mq::Vec2::new(v.x, v.y)
    }
}

impl From<mq::Vec2> for Vec2 {
    fn from(v: mq::Vec2) -> Self {
        Vec2::new(v.x, v.y)
    }
}

impl From<Rect> for mq::Rect {
    fn from(r: Rect) -> Self {
        mq::Rect::new(r.x, r.y, r.w, r.h)
    }
}

impl From<mq::Rect> for Rect {
    fn from(r: mq::Rect) -> Self {
        Rect::new(r.x, r.y, r.w, r.h)
    }
}
