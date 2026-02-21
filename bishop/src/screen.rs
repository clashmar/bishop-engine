//! Screen trait for querying window dimensions.

/// Trait for screen/window information.
pub trait Screen {
    /// Returns the current screen/window width in pixels.
    fn screen_width(&self) -> f32;

    /// Returns the current screen/window height in pixels.
    fn screen_height(&self) -> f32;
}
