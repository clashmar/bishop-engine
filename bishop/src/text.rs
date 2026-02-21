use crate::types::*;

/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone)]
pub struct TextParams<'a> {
    pub font: Option<&'a macroquad::text::Font>,
    /// Base size for character height. The size in pixel used during font rasterizing.
    pub font_size: u16,
    /// The glyphs sizes actually drawn on the screen will be font_size * font_scale
    /// However with font_scale too different from 1.0 letters may be blurry
    pub font_scale: f32,
    /// Font X axis would be scaled by font_scale * font_scale_aspect
    /// and Y axis would be scaled by font_scale
    /// Default is 1.0
    pub font_scale_aspect: f32,
    /// Text rotation in radian
    /// Default is 0.0
    pub rotation: f32,
    pub color: Color,
}

impl<'a> Default for TextParams<'a> {
    fn default() -> TextParams<'a> {
        TextParams {
            font: None,
            font_size: 20,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            color: Color::WHITE,
            rotation: 0.0,
        }
    }
}

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
