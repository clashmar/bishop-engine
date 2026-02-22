//! Backend time and frame functions.

#[cfg(feature = "macroquad")]
mod macroquad_time {
    use macroquad::prelude as mq;

    /// Awaits the next frame.
    pub async fn next_frame() {
        mq::next_frame().await
    }

    /// Returns the time elapsed since the last frame in seconds.
    pub fn get_frame_time() -> f32 {
        mq::get_frame_time()
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_time::*;
