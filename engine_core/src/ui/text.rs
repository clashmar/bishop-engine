// engine_core/src/ui/text.rs
use macroquad::prelude::*;
use crate::assets::core_assets::GNF_FONT;

/// Wrapper for draw_text_ex which uses the editor font and scale.
pub fn draw_text_ui(
    text: &str, 
    x: f32, 
    y: f32, 
    font_size: f32, 
    color: Color
) -> TextDimensions {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(&GNF_FONT),
            font_size: font_size as u16,
            color: color,
            ..Default::default()
        },
    )
}

/// Wrapper for measure_text which uses the editor font.
pub fn measure_text_ui(
    text: &str,
    font_size: f32,
    font_scale: f32,
) -> TextDimensions {
    measure_text(text, Some(&GNF_FONT), font_size as u16, font_scale)
}

/// Returns the x position and width for text to be centered on a given x position.
pub fn center_text(x: f32, text: &str, font_size: f32) -> (f32, f32) {
    let text_size = measure_text_ui(text, font_size, 1.0);
    let new_x = x - (text_size.width / 2.);
    (new_x, text_size.width)
}