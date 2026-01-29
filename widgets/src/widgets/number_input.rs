use macroquad::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use crate::{
    WidgetId, measure_text_ui, draw_input_field_text,
    INPUT_NUMBER_STATE, INPUT_FOCUSED, is_dropdown_open,
    FIELD_BACKGROUND_COLOR, OUTLINE_COLOR, DEFAULT_FONT_SIZE_16,
};

/// A numeric input widget using the builder pattern.
///
/// Supports any numeric type that implements `FromStr`, `Display`, `Default`, `Copy`, and `PartialEq`.
pub struct NumberInput<T> {
    id: WidgetId,
    rect: Rect,
    current: T,
    blocked: bool,
}

impl<T> NumberInput<T>
where
    T: FromStr + Display + Default + Copy + PartialEq,
    <T as FromStr>::Err: std::fmt::Debug,
{
    /// Creates a new number input with the given id, rect, and current value.
    pub fn new(id: WidgetId, rect: Rect, current: T) -> Self {
        Self {
            id,
            rect,
            current,
            blocked: false,
        }
    }

    /// Sets whether the input is blocked from interaction.
    pub fn blocked(mut self, blocked: bool) -> Self {
        self.blocked = blocked;
        self
    }

    /// Draws the widget and returns the current numeric value.
    pub fn show(self) -> T {
        let mut text = self.current.to_string();
        let mut cursor_char = text.len();
        let mut focused = false;

        INPUT_NUMBER_STATE.with(|s| {
            let mut map = s.borrow_mut();
            if let Some((saved_text, saved_cur, saved_foc)) = map.get(&self.id) {
                text = saved_text.clone();
                cursor_char = *saved_cur;
                focused = *saved_foc;
            } else {
                cursor_char = text.chars().count();
                map.insert(self.id, (text.clone(), cursor_char, focused));
            }
        });

        if !focused {
            if text.parse::<T>().unwrap_or_default() != self.current {
                text = self.current.to_string();
                cursor_char = text.len();
            }
        }

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, FIELD_BACKGROUND_COLOR);
        draw_rectangle_lines(self.rect.x, self.rect.y, self.rect.w, self.rect.h, 2., WHITE);
        let placeholder = "<#>";
        let display = if text.is_empty() { placeholder } else { &text };

        draw_input_field_text(display, self.rect);

        if is_dropdown_open() {
            return self.current;
        }

        let mouse = mouse_position();
        let mouse_over = self.rect.contains(vec2(mouse.0, mouse.1));

        if is_mouse_button_pressed(MouseButton::Left) {
            focused = mouse_over && !self.blocked;
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
            let caret_x = self.rect.x + 5. + measure_text_ui(prefix, DEFAULT_FONT_SIZE_16, 1.0).width;
            draw_line(
                caret_x,
                self.rect.y + self.rect.h * 0.3,
                caret_x,
                self.rect.y + self.rect.h * 0.8,
                2.,
                OUTLINE_COLOR,
            );
        }

        INPUT_NUMBER_STATE.with(|s| {
            let mut map = s.borrow_mut();
            map.insert(self.id, (text.clone(), cursor_char, focused));
        });

        text.parse::<T>().unwrap_or(self.current)
    }
}

/// Resets the number input state for the given widget id.
pub fn number_input_reset(id: WidgetId) {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.remove(&id);
    });
}
