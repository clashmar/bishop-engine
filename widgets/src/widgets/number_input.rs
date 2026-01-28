use macroquad::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use crate::{
    WidgetId, measure_text_ui, draw_input_field_text,
    INPUT_NUMBER_STATE, INPUT_FOCUSED, is_dropdown_open,
    FIELD_BACKGROUND_COLOR, OUTLINE_COLOR, DEFAULT_FONT_SIZE_16,
};

pub fn gui_input_number_i32(id: WidgetId, rect: Rect, current: i32, blocked: bool) -> i32 {
    gui_input_number_generic(id, rect, current, blocked)
}

pub fn gui_input_number_f32(id: WidgetId, rect: Rect, current: f32, blocked: bool) -> f32 {
    gui_input_number_generic(id, rect, current, blocked)
}

pub fn gui_input_number_generic<T>(
    id: WidgetId,
    rect: Rect,
    current: T,
    blocked: bool,
) -> T
where
    T: FromStr + Display + Default + Copy + PartialEq,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let mut text = current.to_string();
    let mut cursor_char = text.len();
    let mut focused = false;

    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        if let Some((saved_text, saved_cur, saved_foc)) = map.get(&id) {
            text = saved_text.clone();
            cursor_char = *saved_cur;
            focused = *saved_foc;
        } else {
            cursor_char = text.chars().count();
            map.insert(id, (text.clone(), cursor_char, focused));
        }
    });

    if !focused {
        if text.parse::<T>().unwrap_or_default() != current {
            text = current.to_string();
            cursor_char = text.len();
        }
    }

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD_BACKGROUND_COLOR);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);
    let placeholder = "<#>";
    let display = if text.is_empty() { placeholder } else { &text };

    draw_input_field_text(display, rect);

    if is_dropdown_open() {
        return current;
    }

    let mouse = mouse_position();
    let mouse_over = rect.contains(vec2(mouse.0, mouse.1));

    if is_mouse_button_pressed(MouseButton::Left) {
        focused = mouse_over && !blocked;
    }

    if focused {
        INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);

        if is_key_pressed(KeyCode::Backspace) && cursor_char > 0 {
            text.remove(cursor_char - 1);
            cursor_char -= 1;
        }
        if is_key_pressed(KeyCode::Delete) && cursor_char < text.len() {
            text.remove(cursor_char);
        }
        if is_key_pressed(KeyCode::Left) && cursor_char > 0 {
            cursor_char -= 1;
        }
        if is_key_pressed(KeyCode::Right) && cursor_char < text.len() {
            cursor_char += 1;
        }

        while let Some(chr) = get_char_pressed() {
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);

            if chr.is_control() {
                continue;
            }

            if chr == '-' && cursor_char == 0 && !text.starts_with('-') && T::from_str("-0").is_ok() {
                text.insert(cursor_char, chr);
                cursor_char += 1;
                continue;
            }

            let is_float = T::from_str("0.0").is_ok();
            if chr == '.' && is_float && !text.contains('.') {
                text.insert(cursor_char, chr);
                cursor_char += 1;
                continue;
            }

            if chr.is_ascii_digit() {
                text.insert(cursor_char, chr);
                cursor_char += 1;
            }
        }

        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
            focused = false;
        }
    }

    let now = get_time();
    if focused && ((now * 2.0) as i32 % 2 == 0) {
        let prefix = &text[..cursor_char];
        let caret_x = rect.x + 5. + measure_text_ui(prefix, DEFAULT_FONT_SIZE_16, 1.0).width;
        draw_line(
            caret_x,
            rect.y + rect.h * 0.3,
            caret_x,
            rect.y + rect.h * 0.8,
            2.,
            OUTLINE_COLOR,
        );
    }

    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.insert(id, (text.clone(), cursor_char, focused));
    });

    text.parse::<T>().unwrap_or(current)
}

pub fn gui_input_number_reset(id: WidgetId) {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.remove(&id);
    });
}
