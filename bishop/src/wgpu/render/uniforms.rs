//! Uniform buffer types for shaders.

use bytemuck::{Pod, Zeroable};
use crate::camera::Camera2D;

/// Camera uniforms for 2D rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniforms {
    /// Orthographic projection matrix.
    pub projection: [[f32; 4]; 4],
}

impl CameraUniforms {
    /// Creates screen-space projection (origin at top-left, y increases downward).
    pub fn screen_space(width: f32, height: f32) -> Self {
        let projection = glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);
        Self {
            projection: projection.to_cols_array_2d(),
        }
    }

    /// Creates projection from a Camera2D.
    pub fn from_camera2d(camera: &Camera2D, width: f32, height: f32) -> Self {
        let half_w = width / (2.0 * camera.zoom.x);
        let half_h = height / (2.0 * camera.zoom.y);

        let left = camera.target.x - half_w + camera.offset.x / camera.zoom.x;
        let right = camera.target.x + half_w + camera.offset.x / camera.zoom.x;
        let top = camera.target.y - half_h + camera.offset.y / camera.zoom.y;
        let bottom = camera.target.y + half_h + camera.offset.y / camera.zoom.y;

        let mut projection = glam::Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0);

        if camera.rotation != 0.0 {
            let rotation =
                glam::Mat4::from_rotation_z(-camera.rotation.to_radians());
            projection = projection * rotation;
        }

        Self {
            projection: projection.to_cols_array_2d(),
        }
    }
}

impl Default for CameraUniforms {
    fn default() -> Self {
        Self::screen_space(800.0, 600.0)
    }
}
