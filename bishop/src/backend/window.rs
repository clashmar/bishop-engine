//! Backend window and screen functions.

#[cfg(feature = "macroquad")]
mod macroquad_window {
    use crate::types::Color;
    use crate::window::CursorIcon;
    use macroquad::prelude as mq;

    /// Returns the current screen/window width in pixels.
    pub fn screen_width() -> f32 {
        mq::screen_width()
    }

    /// Returns the current screen/window height in pixels.
    pub fn screen_height() -> f32 {
        mq::screen_height()
    }

    /// Clears the screen with the given color.
    pub fn clear_background(color: Color) {
        mq::clear_background(color.into());
    }

    /// Sets the mouse cursor icon.
    pub fn set_cursor_icon(icon: CursorIcon) {
        use macroquad::miniquad::window::set_mouse_cursor;
        set_mouse_cursor(icon.into());
    }

    /// Draws the current FPS in the top-left corner of the screen.
    pub fn draw_fps() {
        mq::draw_fps();
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_window::*;
