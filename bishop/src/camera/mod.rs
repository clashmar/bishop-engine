//! Camera module for controlling the rendering viewport.

mod camera2d;

pub use camera2d::*;

use glam::Vec2;

/// Trait for camera operations.
pub trait Camera {
    /// Sets the active camera for rendering.
    fn set_camera(&mut self, camera: &Camera2D);

    /// Resets to the default screen-space camera.
    fn set_default_camera(&mut self);

    /// Converts screen coordinates to world coordinates using the given camera.
    fn screen_to_world(&self, camera: &Camera2D, screen_pos: Vec2) -> Vec2;
}
