//! Window and screen dimension operations.

mod cursor_icon;
mod window_config;

pub use cursor_icon::*;
pub use window_config::*;

/// Trait for screen/window information and control.
pub trait Window {
    /// Returns the current screen/window width in logical pixels.
    fn screen_width(&self) -> f32;

    /// Returns the current screen/window height in logical pixels.
    fn screen_height(&self) -> f32;

    /// Sets the mouse cursor icon.
    fn set_cursor_icon(&mut self, icon: CursorIcon);

    /// Toggles fullscreen mode and returns the new state.
    fn toggle_fullscreen(&mut self) -> bool;

    /// Returns whether the window is currently in fullscreen mode.
    fn is_fullscreen(&self) -> bool;

    /// Returns the display scale factor (DPI scaling).
    fn scale_factor(&self) -> f32;
}
