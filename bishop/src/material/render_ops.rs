//! Render operations trait for offscreen render targets.

use super::BishopRenderTarget;

/// Trait for render target operations (offscreen rendering and drawing).
pub trait RenderOps {
    /// Redirects rendering to an offscreen render target.
    fn begin_render_to_target(&mut self, rt: &BishopRenderTarget);

    /// Stops rendering to the offscreen target and restores the screen surface.
    fn end_render_to_target(&mut self);

    /// Draws a render target's contents as a textured quad at the given position and size.
    fn draw_render_target(&mut self, rt: &BishopRenderTarget, x: f32, y: f32, w: f32, h: f32);

    /// Creates a render target compatible with the texture renderer for drawing via `draw_render_target`.
    fn create_drawable_render_target(&self, width: u32, height: u32) -> BishopRenderTarget;
}
