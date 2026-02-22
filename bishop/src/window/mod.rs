//! Window and screen dimension operations.

mod cursor_icon;

pub use cursor_icon::*;

/// Trait for screen/window information.
pub trait Window {
    /// Returns the current screen/window width in pixels.
    fn screen_width(&self) -> f32;

    /// Returns the current screen/window height in pixels.
    fn screen_height(&self) -> f32;
}
