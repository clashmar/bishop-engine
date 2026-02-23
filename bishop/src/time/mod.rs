//! Time and frame timing operations.

/// Trait for frame timing and operations.
pub trait Time {
    /// Returns the time elapsed since the last frame in seconds.
    fn get_frame_time(&self) -> f32;

    /// Called at the start of each frame to update internal state.
    fn update(&mut self);
}
