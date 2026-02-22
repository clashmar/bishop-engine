//! 2D camera for controlling the viewport.

use glam::{Mat4, Vec2, vec2, vec3};
use crate::types::Rect;

/// 2D camera for controlling the viewport.
#[derive(Clone, Debug)]
pub struct Camera2D {
    /// The point in world space the camera is looking at.
    pub target: Vec2,
    /// Zoom level (higher = more zoomed in).
    pub zoom: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
    /// Offset from the target in screen space.
    pub offset: Vec2,
    /// Optional render target for off-screen rendering.
    #[cfg(feature = "macroquad")]
    pub render_target: Option<crate::material::RenderTarget>,
    pub viewport: Option<(i32, i32, i32, i32)>,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            target: Vec2::ZERO,
            zoom: Vec2::ONE,
            rotation: 0.0,
            offset: Vec2::ZERO,
            #[cfg(feature = "macroquad")]
            render_target: None,
            viewport: None,
        }
    }
}

impl Camera2D {
    /// Creates a new camera with the given target and zoom.
    pub fn new(target: Vec2, zoom: Vec2) -> Self {
        Self {
            target,
            zoom,
            ..Default::default()
        }
    }

    /// Converts a world position to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let x = (world_pos.x - self.target.x) * self.zoom.x + self.offset.x;
        let y = (world_pos.y - self.target.y) * self.zoom.y + self.offset.y;
        Vec2::new(x, y)
    }

    /// Returns the world space position for a 2d camera screen space position.
    pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
        let dims = self
            .viewport()
            .map(|(vx, vy, vw, vh)| Rect {
                x: vx as f32,
                y: crate::backend::screen_height() - (vy + vh) as f32,
                w: vw as f32,
                h: vh as f32,
            })
            .unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                w: crate::backend::screen_width(),
                h: crate::backend::screen_height(),
            });

        let point = vec2(
            (point.x - dims.x) / dims.w * 2. - 1.,
            1. - (point.y - dims.y) / dims.h * 2.,
        );
        let inv_mat = self.matrix().inverse();
        let transform = inv_mat.transform_point3(vec3(point.x, point.y, 0.));

        vec2(transform.x, transform.y)
    }

    fn matrix(&self) -> Mat4 {
        let mat_origin = Mat4::from_translation(vec3(-self.target.x, -self.target.y, 0.0));
        let mat_rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        #[cfg(feature = "macroquad")]
        let invert_y = if self.render_target.is_some() {
            1.0
        } else {
            -1.0
        };
        
        #[cfg(not(feature = "macroquad"))]
        let invert_y = -1.0;

        let mat_scale = Mat4::from_scale(vec3(self.zoom.x, self.zoom.y * invert_y, 1.0));
        let mat_translation = Mat4::from_translation(vec3(self.offset.x, self.offset.y, 0.0));

        mat_translation * ((mat_scale * mat_rotation) * mat_origin)
    }

    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        self.viewport
    }
}
