use macroquad::prelude::*;
use crate::{
    WidgetId, measure_text_ui, draw_input_field_text, byte_offset,
    INPUT_TEXT_STATE, INPUT_FOCUSED, is_dropdown_open,
    FIELD_BACKGROUND_COLOR, OUTLINE_COLOR, DEFAULT_FONT_SIZE_16,
    HOLD_INITIAL_DELAY, HOLD_REPEAT_RATE, PLACEHOLDER_TEXT,
};

/// A text input widget using the builder pattern.
pub struct TextInput<'a> {
    id: WidgetId,
    rect: Rect,
    current: &'a str,
    blocked: bool,
    start_focused: bool,
    max_len: Option<usize>,
}

impl<'a> TextInput<'a> {
    /// Creates a new text input widget with the given id, rect, and current value.
    pub fn new(id: WidgetId, rect: Rect, current: &'a str) -> Self {
        Self {
            id,
            rect,
            current,
            blocked: false,
            start_focused: false,
            max_len: None,
        }
    }

    /// Sets whether the input is blocked from interaction.
    pub fn blocked(mut self, blocked: bool) -> Self {
        self.blocked = blocked;
        self
    }

    /// Sets whether the input should start focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.start_focused = focused;
        self
    }

    /// Sets the maximum character length for the input.
    pub fn max_len(mut self, max_len: usize) -> Self {
        self.max_len = Some(max_len);
        self
    }

    /// Draws the widget and returns the current text and focus state.
    pub fn show(self) -> (String, bool) {
        let mut just_gained_focus = false;

        let mut text = self.current.to_string();
        let mut cursor_char = 0usize;
        let mut focused = false;
        let mut last_backspace = 0.0_f64;
        let mut repeat_started = false;

        INPUT_TEXT_STATE.with(|s| {
            let mut map = s.borrow_mut();

            if let Some((saved_text, saved_cur, saved_foc, saved_time, saved_repeat)) = map.get(&self.id) {
                text = saved_text.clone();
                focused = if self.start_focused { true } else { *saved_foc };
                just_gained_focus = self.start_focused && !*saved_foc;
                cursor_char = if self.start_focused && just_gained_focus { text.chars().count() } else { *saved_cur };
                last_backspace = *saved_time;
                repeat_started = *saved_repeat;
            } else {
                focused = self.start_focused;
                just_gained_focus = self.start_focused;
                cursor_char = text.chars().count();
                map.insert(self.id, (text.clone(), cursor_char, focused, last_backspace, repeat_started));
            }
        });

        if !focused {
            if text != self.current {
                text = self.current.to_string();
                cursor_char = text.len();
            }
        }

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, FIELD_BACKGROUND_COLOR);
        draw_rectangle_lines(self.rect.x, self.rect.y, self.rect.w, self.rect.h, 2., WHITE);
        let display = if text.is_empty() { PLACEHOLDER_TEXT } else { &text };

        draw_input_field_text(display, self.rect);

        let mouse = mouse_position();
        let mouse_over = self.rect.contains(vec2(mouse.0, mouse.1));
        if is_mouse_button_pressed(MouseButton::Left) {
            if !focused && mouse_over {
                just_gained_focus = true;
            }
            focused = mouse_over && !self.blocked;

            if !focused {
                INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
            }
        }

        if just_gained_focus {
            while get_char_pressed().is_some() {}
        }

        if is_dropdown_open() {
            return (text, false)
        }

        if focused {
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);
            let now = get_time();

            if is_key_pressed(KeyCode::Backspace) && cursor_char > 0 {
                let start = byte_offset(&text, cursor_char - 1);
                let end = byte_offset(&text, cursor_char);
                text.drain(start..end);
                cursor_char -= 1;
                last_backspace = now;
                repeat_started = false;
            } else if is_key_down(KeyCode::Backspace) && cursor_char > 0 {
                let elapsed = now - last_backspace;
                if (!repeat_started && elapsed >= HOLD_INITIAL_DELAY)
                    || (repeat_started && elapsed >= HOLD_REPEAT_RATE)
                {
                    let start = byte_offset(&text, cursor_char - 1);
                    let end = byte_offset(&text, cursor_char);
                    text.drain(start..end);
                    cursor_char -= 1;
                    last_backspace = now;
                    repeat_started = true;
                }
            }

            if is_key_pressed(KeyCode::Delete) && cursor_char < text.chars().count() {
                let start = byte_offset(&text, cursor_char);
                let end = byte_offset(&text, cursor_char + 1);
                text.drain(start..end);
            }

            if is_key_pressed(KeyCode::Left) && cursor_char > 0 {
                cursor_char -= 1;
            }

            if is_key_pressed(KeyCode::Right) && cursor_char < text.chars().count() {
                cursor_char += 1;
            }

            while let Some(chr) = get_char_pressed() {
                if chr.is_ascii_graphic() || chr == ' ' {
                    let cur_len = text.chars().count();
                    if self.max_len.map_or(true, |limit| cur_len < limit) {
                        let pos = byte_offset(&text, cursor_char);
                        text.insert(pos, chr);
                        cursor_char += 1;
                    }
                }
            }

            if is_key_pressed(KeyCode::Escape) || is_key_down(KeyCode::Enter) {
                INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
                focused = false;
            }
        }

        let now = get_time();
        if focused && ((now * 2.0) as i32 % 2 == 0) {
            let byte_pos = byte_offset(&text, cursor_char);
            let prefix = &text[..byte_pos];
            let cursor_x = self.rect.x + 5. + measure_text_ui(prefix, DEFAULT_FONT_SIZE_16, 1.0).width;
            draw_line(
                cursor_x,
                self.rect.y + self.rect.h * 0.3,
                cursor_x,
                self.rect.y + self.rect.h * 0.8,
                2.,
                OUTLINE_COLOR,
            );
        }

        INPUT_TEXT_STATE.with(|s| {
            let mut map = s.borrow_mut();
            map.insert(self.id, (text.clone(), cursor_char, focused, last_backspace, repeat_started));
        });

        (text, focused)
    }
}

/// Resets the text input state for the given widget id.
pub fn text_input_reset(id: WidgetId) {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.remove(&id);
    });
}
