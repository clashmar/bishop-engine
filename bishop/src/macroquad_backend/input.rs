//! Backend input functions.

#[cfg(feature = "macroquad")]
mod macroquad_input {
    use crate::input::{KeyCode, MouseButton};
    use macroquad::prelude as mq;
    use std::cell::RefCell;

    thread_local! {
        static CHAR_BUFFER: RefCell<Vec<char>> = RefCell::new(Vec::new());
    }

    /// Updates the input state. Call once per frame before processing input.
    pub fn update() {
        CHAR_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            while let Some(c) = mq::get_char_pressed() {
                buf.push(c);
            }
        });
    }

    /// Returns true if the key is currently held down.
    pub fn is_key_down(key: KeyCode) -> bool {
        mq::is_key_down(key.into())
    }

    /// Returns true if the key was pressed this frame.
    pub fn is_key_pressed(key: KeyCode) -> bool {
        mq::is_key_pressed(key.into())
    }

    /// Returns true if the key was released this frame.
    pub fn is_key_released(key: KeyCode) -> bool {
        mq::is_key_released(key.into())
    }

    /// Returns true if the mouse button is currently held down.
    pub fn is_mouse_button_down(button: MouseButton) -> bool {
        mq::is_mouse_button_down(button.into())
    }

    /// Returns true if the mouse button was pressed this frame.
    pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
        mq::is_mouse_button_pressed(button.into())
    }

    /// Returns true if the mouse button was released this frame.
    pub fn is_mouse_button_released(button: MouseButton) -> bool {
        mq::is_mouse_button_released(button.into())
    }

    /// Returns the current mouse position in screen coordinates.
    pub fn mouse_position() -> (f32, f32) {
        mq::mouse_position()
    }

    /// Returns the mouse wheel scroll delta (horizontal, vertical).
    pub fn mouse_wheel() -> (f32, f32) {
        mq::mouse_wheel()
    }

    /// Returns the mouse delta position since last frame.
    pub fn mouse_delta_position() -> (f32, f32) {
        let pos = mq::mouse_delta_position();
        (pos.x, pos.y)
    }

    /// Returns the time in seconds since the application started.
    pub fn get_time() -> f64 {
        mq::get_time()
    }

    /// Returns characters typed this frame for text input.
    pub fn chars_pressed() -> Vec<char> {
        CHAR_BUFFER.with(|buffer| buffer.borrow().clone())
    }

    /// Consumes and returns the next character pressed, or None if empty.
    pub fn get_char_pressed() -> Option<char> {
        CHAR_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            if buf.is_empty() {
                None
            } else {
                Some(buf.remove(0))
            }
        })
    }

    /// Returns the last key pressed this frame.
    pub fn get_last_key_pressed() -> Option<KeyCode> {
        match mq::get_last_key_pressed() {
            _ => Some(KeyCode::Unknown),
        }
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_input::*;
