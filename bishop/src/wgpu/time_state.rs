//! Time and frame timing state for wgpu backend.

use std::time::Instant;

/// Tracks frame timing information.
pub struct TimeState {
    start_time: Instant,
    last_frame_time: Instant,
    delta_time: f32,
}

impl TimeState {
    /// Creates a new time state starting from now.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_time: 0.0,
        }
    }

    /// Updates timing at the start of each frame.
    pub fn begin_frame(&mut self) {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
    }

    /// Returns the time elapsed since the last frame in seconds.
    pub fn frame_time(&self) -> f32 {
        self.delta_time
    }

    /// Returns the time elapsed since the application started in seconds.
    pub fn elapsed(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Updates timing at the end of a frame (alias for begin_frame).
    pub fn tick(&mut self) {
        self.begin_frame();
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
}
