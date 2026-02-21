//! Frame trait for timing and per-frame operations.

use crate::Color;

/// Trait for frame timing and operations.
pub trait Frame {
    /// Returns the time elapsed since the last frame in seconds.
    fn get_frame_time(&self) -> f32;

    /// Clears the screen with the given color.
    fn clear_background(&mut self, color: Color);
}
