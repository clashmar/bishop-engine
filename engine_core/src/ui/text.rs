use crate::assets::core_assets::GNF_FONT;
use widgets::TextRenderer;
use macroquad::prelude::*;

pub struct GnfTextRenderer;

impl TextRenderer for GnfTextRenderer {
    fn draw_text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(&GNF_FONT),
                font_size: font_size as u16,
                color,
                ..Default::default()
            },
        )
    }

    fn measure_text(&self, text: &str, font_size: f32, font_scale: f32) -> TextDimensions {
        measure_text(text, Some(&GNF_FONT), font_size as u16, font_scale)
    }
}

pub static GNF_TEXT_RENDERER: GnfTextRenderer = GnfTextRenderer;

pub fn draw_text_ui(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(&GNF_FONT),
            font_size: font_size as u16,
            color,
            ..Default::default()
        },
    )
}

pub fn measure_text_ui(text: &str, font_size: f32, font_scale: f32) -> TextDimensions {
    measure_text(text, Some(&GNF_FONT), font_size as u16, font_scale)
}

pub fn center_text(x: f32, text: &str, font_size: f32) -> (f32, f32) {
    let text_size = measure_text_ui(text, font_size, 1.0);
    let new_x = x - (text_size.width / 2.);
    (new_x, text_size.width)
}