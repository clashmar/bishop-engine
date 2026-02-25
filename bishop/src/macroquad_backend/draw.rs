//! Backend drawing functions.

#[cfg(feature = "macroquad")]
mod macroquad_draw {
    use crate::draw::DrawTextureParams;
    use crate::types::{Color, Vec2};
    use macroquad::prelude as mq;

    /// Draws a filled rectangle.
    pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
        mq::draw_rectangle(x, y, w, h, color.into());
    }

    /// Draws a rectangle outline.
    pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        mq::draw_rectangle_lines(x, y, w, h, thickness, color.into());
    }

    /// Draws a line between two points.
    pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        mq::draw_line(x1, y1, x2, y2, thickness, color.into());
    }

    /// Draws a filled circle.
    pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {
        mq::draw_circle(x, y, radius, color.into());
    }

    /// Draws a circle outline.
    pub fn draw_circle_lines(x: f32, y: f32, radius: f32, thickness: f32, color: Color) {
        mq::draw_circle_lines(x, y, radius, thickness, color.into());
    }

    /// Draws a filled triangle.
    pub fn draw_triangle(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
        mq::draw_triangle(
            (v1.x, v1.y).into(),
            (v2.x, v2.y).into(),
            (v3.x, v3.y).into(),
            color.into(),
        );
    }

    /// Clears the screen with the specified color.
    pub fn clear(color: Color) {
        mq::clear_background(color.into());
    }

    /// Draws a texture with extended parameters.
    pub fn draw_texture_ex(texture: &mq::Texture2D, x: f32, y: f32, color: Color, params: DrawTextureParams) {
        mq::draw_texture_ex(
            texture,
            x,
            y,
            color.into(),
            mq::DrawTextureParams {
                dest_size: params.dest_size.map(|v| (v.x, v.y).into()),
                source: params.source.map(|r| r.into()),
                rotation: params.rotation,
                flip_x: params.flip_x,
                flip_y: params.flip_y,
                pivot: params.pivot.map(|v| (v.x, v.y).into()),
            },
        );
    }

    /// Draw a texture.
    pub fn draw_texture(texture: &mq::Texture2D, x: f32, y: f32, color: Color) {
        mq::draw_texture(texture, x, y, color.into());
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_draw::*;
