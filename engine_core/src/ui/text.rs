use bishop::prelude::*;

/// Draws text using the GNF font via bishop backend.
pub fn draw_text_ui(text: &str, x: f32, y: f32, font_size: f32, color: impl Into<Color>) -> TextDimensions {
    draw_text(text, x, y, font_size, color.into())
}

/// Measures text using the GNF font via bishop backend.
pub fn measure_text_ui(text: &str, font_size: f32, _font_scale: f32) -> TextDimensions {
    measure_text(text, font_size)
}

/// Centers text horizontally around a given x position.
pub fn center_text(x: f32, text: &str, font_size: f32) -> (f32, f32) {
    let text_size = measure_text_ui(text, font_size, 1.0);
    let new_x = x - (text_size.width / 2.);
    (new_x, text_size.width)
}
