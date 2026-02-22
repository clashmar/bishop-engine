//! Draw texture parameters.

use crate::types::{Rect, Vec2};

/// Parameters for textured drawing operations.
#[derive(Clone, Debug, Default)]
pub struct DrawTextureParams {
    /// Destination size. If None, uses texture size.
    pub dest_size: Option<Vec2>,
    /// Source rectangle within the texture. If None, uses entire texture.
    pub source: Option<Rect>,
    /// Rotation in radians.
    pub rotation: f32,
    /// Flip horizontally.
    pub flip_x: bool,
    /// Flip vertically.
    pub flip_y: bool,
    /// Rotation pivot point. If None, uses center.
    pub pivot: Option<Vec2>,
}
