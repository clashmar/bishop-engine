//! Text measurement dimensions.

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
