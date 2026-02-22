//! Input handling for keyboard and mouse.

mod keycode;
mod mouse;

pub use keycode::*;
pub use mouse::*;

/// Input state abstraction for keyboard and mouse.
pub trait Input {
    /// Returns true if the key is currently held down.
    fn is_key_down(&self, key: KeyCode) -> bool;

    /// Returns true if the key was pressed this frame.
    fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// Returns true if the key was released this frame.
    fn is_key_released(&self, key: KeyCode) -> bool;

    /// Returns true if the mouse button is currently held down.
    fn is_mouse_button_down(&self, button: MouseButton) -> bool;

    /// Returns true if the mouse button was pressed this frame.
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;

    /// Returns true if the mouse button was released this frame.
    fn is_mouse_button_released(&self, button: MouseButton) -> bool;

    /// Returns the current mouse position in screen coordinates.
    fn mouse_position(&self) -> (f32, f32);

    /// Returns the mouse wheel scroll delta (horizontal, vertical).
    fn mouse_wheel(&self) -> (f32, f32);

    /// Returns characters typed this frame for text input.
    fn chars_pressed(&self) -> Vec<char>;

    /// Returns the time in seconds since the application started.
    fn get_time(&self) -> f64;
}
