use crate::types::*;

/// Core drawing operations for 2D primitives.
pub trait Draw {
    /// Draws a filled rectangle.
    fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color);

    /// Draws a rectangle outline.
    fn draw_rectangle_lines(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color);

    /// Draws a line between two points.
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color);

    /// Draws a filled circle.
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color);

    /// Draws a circle outline.
    fn draw_circle_lines(&mut self, x: f32, y: f32, radius: f32, thickness: f32, color: Color);

    /// Draws a filled triangle.
    fn draw_triangle(&mut self, v1: Vec2, v2: Vec2, v3: Vec2, color: Color);

    /// Clears the screen with the specified color.
    fn clear(&mut self, color: Color);
}

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

/// Extended drawing operations for textures.
pub trait DrawTexture {
    /// Backend-specific texture type.
    type Texture;

    /// Draws a texture at the specified position.
    fn draw_texture(&mut self, texture: &Self::Texture, x: f32, y: f32, color: Color);

    /// Draws a texture with extended parameters.
    fn draw_texture_ex(
        &mut self,
        texture: &Self::Texture,
        x: f32,
        y: f32,
        color: Color,
        params: DrawTextureParams,
    );
}
