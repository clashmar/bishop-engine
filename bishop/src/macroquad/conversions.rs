//! Type conversions between bishop and macroquad types.

use crate::camera::Camera2D;
use crate::input::{KeyCode, MouseButton};
use crate::types::{Color, Rect, Vec2};
use crate::window::CursorIcon;
use macroquad::miniquad;
use macroquad::prelude as mq;

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

impl From<&Camera2D> for mq::Camera2D {
    fn from(cam: &Camera2D) -> Self {
        mq::Camera2D {
            target: (cam.target.x, cam.target.y).into(),
            zoom: (cam.zoom.x, cam.zoom.y).into(),
            rotation: cam.rotation,
            offset: (cam.offset.x, cam.offset.y).into(),
            render_target: cam.render_target.clone(),
            viewport: None,
        }
    }
}

impl From<&mq::Camera2D> for Camera2D {
    fn from(cam: &mq::Camera2D) -> Self {
        Camera2D {
            target: {
                let v = cam.target;
                Vec2::new(v.x, v.y)
            },
            zoom: {
                let v = cam.zoom;
                Vec2::new(v.x, v.y)
            },
            rotation: cam.rotation,
            offset: {
                let v = cam.offset;
                Vec2::new(v.x, v.y)
            },
            render_target: cam.render_target.clone(),
            viewport: cam.viewport,
        }
    }
}

impl From<CursorIcon> for miniquad::CursorIcon {
    fn from(icon: CursorIcon) -> Self {
        match icon {
            CursorIcon::Default => miniquad::CursorIcon::Default,
            CursorIcon::Pointer => miniquad::CursorIcon::Pointer,
            CursorIcon::Crosshair => miniquad::CursorIcon::Crosshair,
            CursorIcon::Move => miniquad::CursorIcon::Move,
            CursorIcon::Text => miniquad::CursorIcon::Text,
            CursorIcon::NotAllowed => miniquad::CursorIcon::NotAllowed,
            CursorIcon::EWResize => miniquad::CursorIcon::EWResize,
            CursorIcon::NSResize => miniquad::CursorIcon::NSResize,
            CursorIcon::NESWResize => miniquad::CursorIcon::NESWResize,
            CursorIcon::NWSEResize => miniquad::CursorIcon::NWSEResize,
        }
    }
}
