//! Backend camera functions.

#[cfg(feature = "macroquad")]
mod macroquad_camera {
    use crate::camera::Camera2D;
    use crate::types::Vec2;
    use macroquad::prelude as mq;

    /// Sets the active camera for rendering.
    pub fn set_camera(camera: &Camera2D) {
        mq::set_camera(&mq::Camera2D::from(camera));
    }

    /// Resets to the default screen-space camera.
    pub fn set_default_camera() {
        mq::set_default_camera();
    }

    /// Converts screen coordinates to world coordinates using the given camera.
    pub fn screen_to_world(camera: &Camera2D, screen_pos: Vec2) -> Vec2 {
        let mq_cam = mq::Camera2D::from(camera);
        let mq_world: mq::Vec2 = mq_cam.screen_to_world((screen_pos.x, screen_pos.y).into());
        Vec2::new(mq_world.x, mq_world.y)
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_camera::*;
