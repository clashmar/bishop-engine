use macroquad::prelude::*;
use crate::{
    draw_text_ui, measure_text_ui,
    DEFAULT_FONT_SIZE_16, FIELD_TEXT_COLOR, WIDGET_PADDING, PLACEHOLDER_TEXT
};

pub fn byte_offset(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or_else(|| s.len())
}

pub fn draw_input_field_text(text: &str, rect: Rect) {
    draw_text_ui(
        text,
        rect.x + WIDGET_PADDING / 2.,
        rect.y + rect.h * 0.7,
        DEFAULT_FONT_SIZE_16,
        FIELD_TEXT_COLOR,
    );
}

pub fn center_text_field(x: f32, text: &str) -> (f32, f32) {
    let text_to_measure = if text.is_empty() { PLACEHOLDER_TEXT } else { text };
    let text_size = measure_text_ui(text_to_measure, DEFAULT_FONT_SIZE_16, 1.0);
    let new_x = x - (text_size.width / 2.);
    (new_x - WIDGET_PADDING / 2., text_size.width + WIDGET_PADDING)
}

pub fn rect_width_for_text(text: &str, font_size: f32) -> f32 {
    measure_text_ui(text, font_size, 1.0).width + WIDGET_PADDING * 2.0
}
