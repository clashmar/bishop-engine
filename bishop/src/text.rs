use crate::types::*;

/// Dimensions returned from text measurement and rendering.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextDimensions {
    /// Width of the text in pixels.
    pub width: f32,
    /// Height of the text in pixels.
    pub height: f32,
    /// Vertical offset from baseline.
    pub offset_y: f32,
}

/// Text rendering and measurement operations.
pub trait Text {
    /// Draws text at the specified position and returns its dimensions.
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions;

    /// Measures text without drawing it.
    fn measure_text(&self, text: &str, font_size: f32) -> TextDimensions;
}
