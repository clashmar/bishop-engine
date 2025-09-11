// engine_core/src/ui/widgets.rs
use macroquad::prelude::*;
use std::collections::HashMap;
use std::cell::RefCell;

/// Editable text field. Returns the current contents.
/// The widget keeps focus until the user clicks outside the rectangle
/// (or presses <kbd>Esc</kbd>) and shows a blinking cursor while active.
pub fn gui_input_text(rect: Rect, current: &str) -> String {
    thread_local! {
        static STATE: RefCell<HashMap<(i32, i32, i32, i32), (String, usize, bool)>> =
            RefCell::new(HashMap::new());
    }

    // Load / initialise widget state
    let mut text = current.to_string();
    let mut cursor_char = 0usize;   // cursor expressed in *characters*
    let mut focused = false;

    STATE.with(|s| {
        let mut map = s.borrow_mut();
        let key = (
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.w.round() as i32,
            rect.h.round() as i32,
        );
        if let Some((saved, saved_cur, saved_foc)) = map.get(&key) {
            text = saved.clone();
            cursor_char = *saved_cur;
            focused = *saved_foc;
        } else {
            map.insert(key, (text.clone(), cursor_char, focused));
        }
    });

    // Draw background & current text
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);

    let placeholder = "<type here>";
    let display = if text.is_empty() { placeholder } else { &text };
    draw_text_ex(
        display,
        rect.x + 5.,
        rect.y + rect.h * 0.7,
        TextParams {
            font_size: 20,
            color: WHITE,
            ..Default::default()
        },
    );

    // Focus handling
    let mouse = mouse_position();
    let mouse_over = rect.contains(vec2(mouse.0, mouse.1));
    if is_mouse_button_pressed(MouseButton::Left) {
        focused = mouse_over;
    }

    // Keyboard input (only when focused)
    if focused {
        // Backspace 
        if is_key_pressed(KeyCode::Backspace) && cursor_char > 0 {
            let start = byte_offset(&text, cursor_char - 1);
            let end   = byte_offset(&text, cursor_char);
            text.drain(start..end);
            cursor_char -= 1;
        }

        // Delete 
        if is_key_pressed(KeyCode::Delete) && cursor_char < text.chars().count() {
            let start = byte_offset(&text, cursor_char);
            let end   = byte_offset(&text, cursor_char + 1);
            text.drain(start..end);
        }

        // Arrow keys (move cursor)
        if is_key_pressed(KeyCode::Left) && cursor_char > 0 {
            cursor_char -= 1;
        }
        if is_key_pressed(KeyCode::Right) && cursor_char < text.chars().count() {
            cursor_char += 1;
        }

        // Typed characters
        while let Some(chr) = get_char_pressed() {
            // Accept only printable ASCII characters
            if chr.is_ascii_graphic() {
                let pos = byte_offset(&text, cursor_char);
                text.insert(pos, chr);
                cursor_char += 1;
            }
        }

        // Escape
        if is_key_pressed(KeyCode::Escape) {
            focused = false;
        }
    }

    // Blinking cursor
    let now = get_time();
    if focused && ((now * 2.0) as i32 % 2 == 0) {
        // Convert the *character* cursor to a byte offset for slicing
        let byte_pos = byte_offset(&text, cursor_char);
        let prefix = &text[..byte_pos];
        let cursor_x = rect.x + 5. + measure_text(prefix, None, 20, 1.0).width;
        draw_line(
            cursor_x,
            rect.y + rect.h * 0.3,
            cursor_x,
            rect.y + rect.h * 0.8,
            2.,
            WHITE,
        );
    }

    // Persist state
    STATE.with(|s| {
        let mut map = s.borrow_mut();
        let key = (
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.w.round() as i32,
            rect.h.round() as i32,
        );
        map.insert(key, (text.clone(), cursor_char, focused));
    });

    text
}

/// Simple toggle widget. Returns `true` when the value changed this frame.
pub fn gui_checkbox(rect: Rect, value: &mut bool) -> bool {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);

    if *value {
        draw_line(
            rect.x + 3.,
            rect.y + rect.h * 0.5,
            rect.x + rect.w * 0.4,
            rect.y + rect.h - 4.,
            2.,
            GREEN,
        );
        draw_line(
            rect.x + rect.w * 0.4,
            rect.y + rect.h - 4.,
            rect.x + rect.w - 3.,
            rect.y + 4.,
            2.,
            GREEN,
        );
    }

    let mouse = mouse_position();
    if is_mouse_button_pressed(MouseButton::Left) && rect.contains(vec2(mouse.0, mouse.1)) {
        *value = !*value;
        true
    } else {
        false
    }
}

/// Rectangular button with a centered label. Returns `true` on click.
pub fn gui_button(rect: Rect, label: &str) -> bool {
    let mouse = mouse_position();
    let hovered = rect.contains(vec2(mouse.0, mouse.1));
    let bg = if hovered {
        Color::new(0.2, 0.2, 0.2, 0.8)
    } else {
        Color::new(0., 0., 0., 0.6)
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);

    let txt_dims = measure_text(label, None, 20, 1.0);
    let txt_x = rect.x + (rect.w - txt_dims.width) / 2.;
    let txt_y = rect.y + rect.h * 0.7;
    draw_text(label, txt_x, txt_y, 20., WHITE);

    is_mouse_button_pressed(MouseButton::Left) && hovered
}

/// Numeric field that accepts only digits, a single decimal point and an
/// optional leading minus sign. Returns the parsed `f32`; on parse error the
/// original `current` value is returned.
pub fn gui_input_number(rect: Rect, current: f32) -> f32 {
    use std::f32::EPSILON;

    thread_local! {
        static STATE: RefCell<HashMap<(i32, i32, i32, i32), (String, usize, bool)>> =
            RefCell::new(HashMap::new());
    }

    // Load or initialise state
    let mut txt = current.to_string();
    let mut cursor = txt.len(); // Place the cursor at the end
    let mut focused = false;

    STATE.with(|s| {
        let mut map = s.borrow_mut();
        let key = (
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.w.round() as i32,
            rect.h.round() as i32,
        );
        // If we already have a state entry, use it.
        if let Some((saved_txt, saved_cur, saved_foc)) = map.get(&key) {
            txt = saved_txt.clone();
            cursor = *saved_cur;
            focused = *saved_foc;
        } else {
            // First time we see this widget store the initial state.
            map.insert(key, (txt.clone(), cursor, focused));
        }
    });

    // If the widget is not focused, force‑sync the displayed
    // text with the latest current value.
    if !focused {
        // Only replace when the numeric value actually differs. This
        // avoids flickering the cursor position when the user is typing.
        if (txt.parse::<f32>().unwrap_or(0.0) - current).abs() > EPSILON {
            txt = current.to_string();
            cursor = txt.len(); // put cursor at the end of the new text
        }
    }

    // Draw background and current text
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);
    let placeholder = "<type number>";
    let display = if txt.is_empty() { placeholder } else { &txt };
    draw_text_ex(
        display,
        rect.x + 5.,
        rect.y + rect.h * 0.7,
        TextParams {
            font_size: 20,
            color: WHITE,
            ..Default::default()
        },
    );

    // Focus handling
    let mouse = mouse_position();
    let mouse_over = rect.contains(vec2(mouse.0, mouse.1));
    if is_mouse_button_pressed(MouseButton::Left) {
        focused = mouse_over;
    }

    // Keyboard input
    if focused {
        if is_key_pressed(KeyCode::Backspace) && cursor > 0 {
            txt.remove(cursor - 1);
            cursor -= 1;
        }
        if is_key_pressed(KeyCode::Delete) && cursor < txt.len() {
            txt.remove(cursor);
        }
        if is_key_pressed(KeyCode::Left) && cursor > 0 {
            cursor -= 1;
        }
        if is_key_pressed(KeyCode::Right) && cursor < txt.len() {
            cursor += 1;
        }

        while let Some(chr) = get_char_pressed() {
            if chr.is_control() {
                continue;
            }
            // Leading minus
            if chr == '-' && cursor == 0 && !txt.starts_with('-') {
                txt.insert(cursor, chr);
                cursor += 1;
                continue;
            }
            // Single decimal point
            if chr == '.' && !txt.contains('.') {
                txt.insert(cursor, chr);
                cursor += 1;
                continue;
            }
            // Digits
            if chr.is_ascii_digit() {
                txt.insert(cursor, chr);
                cursor += 1;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            focused = false;
        }
    }

    // Blinking cursor
    let now = get_time();
    if focused && ((now * 2.0) as i32 % 2 == 0) {
        let prefix = &txt[..cursor];
        let cursor_x = rect.x + 5. + measure_text(prefix, None, 20, 1.0).width;
        draw_line(
            cursor_x,
            rect.y + rect.h * 0.3,
            cursor_x,
            rect.y + rect.h * 0.8,
            2.,
            WHITE,
        );
    }

    // Persist state
    STATE.with(|s| {
        let mut map = s.borrow_mut();
        let key = (
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.w.round() as i32,
            rect.h.round() as i32,
        );
        map.insert(key, (txt.clone(), cursor, focused));
    });

    txt.parse::<f32>().unwrap_or(current)
}

/// Returns the byte offset of the `char_idx`‑th character in `s`.
/// If `char_idx` is out of range, returns `s.len()`.
fn byte_offset(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or_else(|| s.len())
}