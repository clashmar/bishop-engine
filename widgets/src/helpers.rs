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

/// Returns the selection range as (start, end) where start <= end.
pub fn selection_range(cursor: usize, anchor: Option<usize>) -> Option<(usize, usize)> {
    anchor.map(|a| {
        if cursor < a {
            (cursor, a)
        } else {
            (a, cursor)
        }
    })
}

/// Gets the selected text from a string given cursor position and optional anchor.
pub fn get_selected_text(text: &str, cursor: usize, anchor: Option<usize>) -> Option<String> {
    selection_range(cursor, anchor).map(|(start, end)| {
        let start_byte = byte_offset(text, start);
        let end_byte = byte_offset(text, end);
        text[start_byte..end_byte].to_string()
    })
}

/// Deletes the selected text and returns the new cursor position.
pub fn delete_selection(text: &mut String, cursor: usize, anchor: Option<usize>) -> usize {
    if let Some((start, end)) = selection_range(cursor, anchor) {
        let start_byte = byte_offset(text, start);
        let end_byte = byte_offset(text, end);
        text.drain(start_byte..end_byte);
        start
    } else {
        cursor
    }
}

/// Filters pasted text for numeric input, keeping only valid numeric characters.
pub fn filter_numeric_paste(input: &str, is_float: bool, allow_negative: bool, has_decimal: bool) -> String {
    let mut result = String::new();
    let mut seen_decimal = has_decimal;

    for (i, ch) in input.chars().enumerate() {
        if ch == '-' && i == 0 && allow_negative && result.is_empty() {
            result.push(ch);
        } else if ch == '.' && is_float && !seen_decimal {
            result.push(ch);
            seen_decimal = true;
        } else if ch.is_ascii_digit() {
            result.push(ch);
        }
    }

    result
}

/// Calculates the character index from a mouse x-coordinate within the text field.
pub fn char_index_from_x(text: &str, mouse_x: f32, field_x: f32, font_size: f32) -> usize {
    let text_start_x = field_x + WIDGET_PADDING / 2.;
    let relative_x = mouse_x - text_start_x;

    if relative_x <= 0.0 {
        return 0;
    }

    let mut prev_width = 0.0;
    for (i, _) in text.char_indices() {
        let char_idx = text[..i].chars().count();
        let prefix = &text[..i];
        let width = measure_text_ui(prefix, font_size, 1.0).width;

        if relative_x < width {
            let mid = (prev_width + width) / 2.0;
            if relative_x < mid {
                return char_idx.saturating_sub(1);
            } else {
                return char_idx;
            }
        }
        prev_width = width;
    }

    text.chars().count()
}
