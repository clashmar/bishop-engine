use macroquad::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use crate::*;
use crate::clipboard::{clipboard_get_text, clipboard_set_text};

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
        let is_float = T::from_str("0.0").is_ok();
        let allow_negative = T::from_str("-0").is_ok();

        let (mut text, mut cursor_char, mut focused, mut selection_anchor, mut last_key_time, mut repeat_key, mut repeat_started) =
            INPUT_NUMBER_STATE.with(|s| {
                let mut map = s.borrow_mut();
                if let Some(state) = map.get(&self.id) {
                    (state.text.clone(), state.cursor_char, state.focused, state.selection_anchor, state.last_key_time, state.repeat_key, state.repeat_started)
                } else {
                    let t = self.current.to_string();
                    let cc = t.chars().count();
                    map.insert(self.id, NumberInputState::new(t.clone()));
                    (t, cc, false, None, 0.0, None, false)
                }
            });

        if !focused && text.parse::<T>().unwrap_or_default() != self.current {
            text = self.current.to_string();
            cursor_char = text.chars().count();
        }

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, FIELD_BACKGROUND_COLOR);
        draw_rectangle_lines(self.rect.x, self.rect.y, self.rect.w, self.rect.h, 2., WHITE);

        if let Some((start, end)) = selection_range(cursor_char, selection_anchor) {
            let start_x = self.rect.x + WIDGET_PADDING / 2. + measure_text_ui(&text[..start], DEFAULT_FONT_SIZE_16, 1.0).width;
            let end_x = self.rect.x + WIDGET_PADDING / 2. + measure_text_ui(&text[..end], DEFAULT_FONT_SIZE_16, 1.0).width;
            draw_rectangle(
                start_x,
                self.rect.y + self.rect.h * 0.2,
                end_x - start_x,
                self.rect.h * 0.6,
                Color::new(0.3, 0.5, 0.8, 0.5),
            );
        }

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
            if focused {
                selection_anchor = None;
            }
        }

        if focused {
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);
            let now = get_time();
            let shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            let ctrl_held = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);

            if ctrl_held && is_key_pressed(KeyCode::A) {
                selection_anchor = Some(0);
                cursor_char = text.chars().count();
            }

            if ctrl_held && is_key_pressed(KeyCode::C) {
                if let Some(selected) = get_selected_text(&text, cursor_char, selection_anchor) {
                    clipboard_set_text(&selected);
                }
            }

            if ctrl_held && is_key_pressed(KeyCode::V) {
                if let Some(clipboard_text) = clipboard_get_text() {
                    let insert_pos = if selection_anchor.is_some() {
                        cursor_char = delete_selection(&mut text, cursor_char, selection_anchor);
                        selection_anchor = None;
                        cursor_char
                    } else {
                        cursor_char
                    };

                    let at_start = insert_pos == 0;
                    let has_decimal = text.contains('.');
                    let filtered = filter_numeric_paste(
                        &clipboard_text,
                        is_float,
                        allow_negative && at_start && !text.starts_with('-'),
                        has_decimal,
                    );

                    text.insert_str(insert_pos, &filtered);
                    cursor_char += filtered.chars().count();
                }
            }

            let handle_key_action = |key: RepeatableKey, pressed: bool, down: bool, rk: &mut Option<RepeatableKey>, rs: &mut bool, lkt: &mut f64| -> bool {
                if pressed {
                    *rk = Some(key);
                    *lkt = now;
                    *rs = false;
                    true
                } else if down && *rk == Some(key) {
                    let elapsed = now - *lkt;
                    if (!*rs && elapsed >= HOLD_INITIAL_DELAY) || (*rs && elapsed >= HOLD_REPEAT_RATE) {
                        *lkt = now;
                        *rs = true;
                        true
                    } else {
                        false
                    }
                } else {
                    if *rk == Some(key) && !down {
                        *rk = None;
                    }
                    false
                }
            };

            if handle_key_action(
                RepeatableKey::Backspace,
                is_key_pressed(KeyCode::Backspace),
                is_key_down(KeyCode::Backspace),
                &mut repeat_key,
                &mut repeat_started,
                &mut last_key_time,
            ) {
                if selection_anchor.is_some() {
                    cursor_char = delete_selection(&mut text, cursor_char, selection_anchor);
                    selection_anchor = None;
                } else if cursor_char > 0 {
                    text.remove(cursor_char - 1);
                    cursor_char -= 1;
                }
            }

            if handle_key_action(
                RepeatableKey::Delete,
                is_key_pressed(KeyCode::Delete),
                is_key_down(KeyCode::Delete),
                &mut repeat_key,
                &mut repeat_started,
                &mut last_key_time,
            ) {
                if selection_anchor.is_some() {
                    cursor_char = delete_selection(&mut text, cursor_char, selection_anchor);
                    selection_anchor = None;
                } else if cursor_char < text.len() {
                    text.remove(cursor_char);
                }
            }

            if handle_key_action(
                RepeatableKey::Left,
                is_key_pressed(KeyCode::Left),
                is_key_down(KeyCode::Left),
                &mut repeat_key,
                &mut repeat_started,
                &mut last_key_time,
            ) && cursor_char > 0 {
                if shift_held {
                    if selection_anchor.is_none() {
                        selection_anchor = Some(cursor_char);
                    }
                } else {
                    selection_anchor = None;
                }
                cursor_char -= 1;
            }

            if handle_key_action(
                RepeatableKey::Right,
                is_key_pressed(KeyCode::Right),
                is_key_down(KeyCode::Right),
                &mut repeat_key,
                &mut repeat_started,
                &mut last_key_time,
            ) && cursor_char < text.len() {
                if shift_held {
                    if selection_anchor.is_none() {
                        selection_anchor = Some(cursor_char);
                    }
                } else {
                    selection_anchor = None;
                }
                cursor_char += 1;
            }

            while let Some(chr) = get_char_pressed() {
                INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);

                if chr.is_control() {
                    continue;
                }

                if selection_anchor.is_some() {
                    cursor_char = delete_selection(&mut text, cursor_char, selection_anchor);
                    selection_anchor = None;
                }

                if chr == '-' && cursor_char == 0 && !text.starts_with('-') && allow_negative {
                    text.insert(cursor_char, chr);
                    cursor_char += 1;
                    continue;
                }

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
                selection_anchor = None;
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
            map.insert(self.id, NumberInputState {
                text: text.clone(),
                cursor_char,
                focused,
                selection_anchor,
                last_key_time,
                repeat_key,
                repeat_started,
            });
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
