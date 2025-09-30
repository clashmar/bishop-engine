// engine_core/src/ui/widgets.rs
use macroquad::prelude::*;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::Display;
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Opaque, never‑changing identifier for a logical UI widget.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WidgetId(pub usize);

impl Default for WidgetId {
    /// Returns a fresh id. Call this when the widget is created.
    fn default() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        WidgetId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

const WIDGET_PADDING: f32 = 20.0;
const HOLD_INITIAL_DELAY: f64 = 0.50;
const HOLD_REPEAT_RATE: f64   = 0.05;

thread_local! {
    static INPUT_TEXT_STATE: RefCell<HashMap<WidgetId, (String, usize, bool, f64, bool)>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    static INPUT_FOCUSED: RefCell<bool> = RefCell::new(false);
}

/// Global flag that tells the rest of the editor whether a character
/// was consumed by a text field this frame.
pub fn input_is_focused() -> bool {
    INPUT_FOCUSED.with(|f| {
        let mut flag = f.borrow_mut();
        let was = *flag;
        *flag = false;
        was
    })
}

thread_local! {
    static DROPDOWN_OPEN: RefCell<bool> = RefCell::new(false);
}

/// Global flag that tells the rest of the editor whether a dropdown
/// is currently open.
pub fn dropdown_is_open() -> bool {
    DROPDOWN_OPEN.with(|f| *f.borrow())
}

/// Editable text field. Returns the current contents.
/// The widget keeps focus until the user clicks outside the rectangle
/// or presses Esc and shows a blinking cursor while active.
pub fn gui_input_text_default(id: WidgetId, rect: Rect, current: &str) -> (String, bool) {
    gui_input_text(id, rect, current, false, None)
}

/// Editable text field that starts focused. Returns the current contents.
/// The widget keeps focus until the user clicks outside the rectangle
/// or presses Esc and shows a blinking cursor while active.
pub fn gui_input_text_focused(id: WidgetId, rect: Rect, current: &str) -> (String, bool) {
    gui_input_text(id, rect, current, true, None)
}

/// Same as `gui_input_text_default` but clamps the tex to `max_len`.
pub fn gui_input_text_clamped(id: WidgetId, rect: Rect, current: &str, max_len: usize) -> (String, bool) {
    gui_input_text(id, rect, current, false, Some(max_len))
}

/// Same as `gui_input_text_focused` but clamps the tex to `max_len`.
pub fn gui_input_text_clamped_focused(id: WidgetId, rect: Rect, current: &str, max_len: usize) -> (String, bool) {
    gui_input_text(id, rect, current, true, Some(max_len))
}

fn gui_input_text(
    id: WidgetId,
    rect: Rect, 
    current: &str, 
    start_focused: bool,
    max_len: Option<usize>,
) -> (String, bool) {
    // Make sure any outstanding inputs are consumed
    let mut just_gained_focus = false;

    // Load / initialise widget state
    let mut text = current.to_string();
    let mut cursor_char = 0usize;
    let mut focused = false;
    let mut last_backspace = 0.0_f64;
    let mut repeat_started = false;

    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();

        if let Some((saved, saved_cur, saved_foc, saved_time, saved_repeat)) = map.get(&id) {
            text = saved.clone();
            cursor_char = if start_focused { text.chars().count() } else { *saved_cur };
            focused = if start_focused { true } else { *saved_foc };
            just_gained_focus = start_focused && !*saved_foc;
            last_backspace = *saved_time;
            repeat_started = *saved_repeat;
        } else {
            focused = start_focused;
            just_gained_focus = start_focused;
            map.insert(id, (text.clone(), cursor_char, focused, last_backspace, repeat_started));
        }
    });

    // Draw background & current text
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 1.0));
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
        // Clicking inside gains focus, clicking elsewhere loses focus
        if !focused && mouse_over {
            just_gained_focus = true;
        }
        focused = mouse_over;
    }

    if just_gained_focus {
        // Discard everything that was typed while the widget was not active
        while let Some(_) = get_char_pressed() {}
    }

    // Don't update the field if a dropdown is open
    if dropdown_is_open() {
        return (text, false)
    }

    // Keyboard input (only when focused)
    if focused {
        // Tell the rest of the editor the field is focused
        INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);
        let now = get_time();

        // Backspace
        if is_key_pressed(KeyCode::Backspace) && cursor_char > 0 {
            let start = byte_offset(&text, cursor_char - 1);
            let end   = byte_offset(&text, cursor_char);
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
                let end   = byte_offset(&text, cursor_char);
                text.drain(start..end);
                cursor_char -= 1;
                last_backspace = now;
                repeat_started = true;
            }
        }
        
        // Delete
        if is_key_pressed(KeyCode::Delete) && cursor_char < text.chars().count() {
            let start = byte_offset(&text, cursor_char);
            let end   = byte_offset(&text, cursor_char + 1);
            text.drain(start..end);
        }

        // Arrow keys
        if is_key_pressed(KeyCode::Left) && cursor_char > 0 {
            cursor_char -= 1;
        }

        if is_key_pressed(KeyCode::Right) && cursor_char < text.chars().count() {
            cursor_char += 1;
        }

        // Typed characters
        while let Some(chr) = get_char_pressed() {
            if chr.is_ascii_graphic() {
                // Enforce the length limit
                let cur_len = text.chars().count();
                if max_len.map_or(true, |limit| cur_len < limit) {
                    let pos = byte_offset(&text, cursor_char);
                    text.insert(pos, chr);
                    cursor_char += 1;
                }
            }
        }

        // Escape 
        if is_key_pressed(KeyCode::Escape) || is_key_down(KeyCode::Enter) {
            focused = false;
        }
    } else {
        INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    }

    // Blinking cursor
    let now = get_time();
    if focused && ((now * 2.0) as i32 % 2 == 0) {
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
    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.insert(id, (text.clone(), cursor_char, focused, last_backspace, repeat_started));
    });

    // Return the current text and whether the widget still has focus
    (text, focused)
}

/// Remove any stored state for the given id.
pub fn gui_input_text_reset(id: WidgetId) {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);

    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.remove(&id);
    });
}

/// Clears the focused flag of all text fields.
pub fn clear_all_text_focus() {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();
        for (_, entry) in map.iter_mut() {
            entry.2 = false;
        }
    });
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

    // Don't update the field if a dropdown is open
    if dropdown_is_open() {
        return *value
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

    // Don't highligh if a dropdown is open
    let bg = if hovered && !dropdown_is_open() {
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

    is_mouse_button_pressed(MouseButton::Left) && 
    hovered && 
    !dropdown_is_open()
}

/// Numeric field that accepts only digits, a single decimal point and an
/// optional leading minus sign. Returns the parsed `f32`; on parse error the
/// original `current` value is returned.
pub fn gui_input_number(
    id: WidgetId,
    rect: Rect,
     current: f32
    ) -> f32 {
    use std::f32::EPSILON;

    thread_local! {
        static STATE: RefCell<HashMap<WidgetId, (String, usize, bool)>> =
            RefCell::new(HashMap::new());
    }

    // Load or initialise state
    let mut txt = current.to_string();
    let mut cursor = txt.len(); // Place the cursor at the end
    let mut focused = false;

    STATE.with(|s| {
        let mut map = s.borrow_mut();

        // If we already have a state entry, use it
        if let Some((saved_txt, saved_cur, saved_foc)) = map.get(&id) {
            txt = saved_txt.clone();
            cursor = *saved_cur;
            focused = *saved_foc;
        } else {
            // First time we see this widget store the initial state.
            map.insert(id, (txt.clone(), cursor, focused));
        }
    });

    // If the widget is not focused, force‑sync the displayed
    // text with the latest current value
    if !focused {
        // Only replace when the numeric value actually differs. This
        // avoids flickering the cursor position when the user is typing
        if (txt.parse::<f32>().unwrap_or(0.0) - current).abs() > EPSILON {
            txt = current.to_string();
            cursor = txt.len(); // Put cursor at the end of the new text
        }
    }

    // Draw background and current text
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);
    let placeholder = "<#>";
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

    if dropdown_is_open() {
        return current;
    }

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
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);
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

        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter)  {
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
        map.insert(id, (txt.clone(), cursor, focused));
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

/// Horizontal slider that returns the new value and a `bool` indicating
/// whether the user moved the handle this frame.
pub fn gui_slider(id: WidgetId, rect: Rect, min: f32, max: f32, value: f32) -> (f32, bool) {
    thread_local! {
        static STATE: RefCell<HashMap<WidgetId, (bool, f32)>> =
            RefCell::new(HashMap::new());
    }

    // Load persisted state
    let mut dragging = false;
    let mut drag_offset = 0.0_f32; // distance mouse → handle left edge
    STATE.with(|s| {
        let map = s.borrow();
        if let Some(&(d, off)) = map.get(&id) {
            dragging = d;
            drag_offset = off;
        }
    });

    // Geometry
    let track_h = rect.h * 0.2;
    let track_y = rect.y + (rect.h - track_h) * 0.5;
    let handle_sz = rect.h; // square handle
    let range = max - min;
    let norm = ((value - min) / range).clamp(0.0, 1.0);
    let handle_x = rect.x + norm * (rect.w - handle_sz);

    // Draw background & handle
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.5));
    draw_rectangle(rect.x, track_y, rect.w, track_h, Color::new(0.2, 0.2, 0.2, 0.8));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);

    let handle_col = if dragging && !dropdown_is_open() {
        Color::new(0.6, 0.6, 0.9, 1.0)
    } else {
        Color::new(0.4, 0.4, 0.8, 1.0)
    };
    draw_rectangle(handle_x, rect.y, handle_sz, rect.h, handle_col);
    draw_rectangle_lines(handle_x, rect.y, handle_sz, rect.h, 2., WHITE);

    if dropdown_is_open() {
        return (value, false)
    }

    // Input handling
    let mouse = mouse_position();
    let mouse_over_handle = Rect::new(handle_x, rect.y, handle_sz, rect.h)
        .contains(vec2(mouse.0, mouse.1));
    let mouse_over_track = rect.contains(vec2(mouse.0, mouse.1));

    // Start dragging
    if is_mouse_button_pressed(MouseButton::Left) && mouse_over_handle {
        dragging = true;
        drag_offset = mouse.0 - handle_x;
    }

    // Release drag on mouse up
    if is_mouse_button_released(MouseButton::Left) {
        dragging = false;
        drag_offset = 0.0;
    }

    // Compute new value
    let mut new_value = value;
    let mut changed = false;

    if dragging {
        // Apply the saved offset so the handle follows the cursor naturally
        let handle_center = mouse.0 - drag_offset;
        let rel = ((handle_center - rect.x) / (rect.w - handle_sz)).clamp(0.0, 1.0);
        new_value = min + rel * range;
        changed = (new_value - value).abs() > f32::EPSILON;
    } else if mouse_over_track && is_mouse_button_pressed(MouseButton::Left) {
        // Click‑on‑track behaviour
        let rel = ((mouse.0 - rect.x) / (rect.w - handle_sz)).clamp(0.0, 1.0);
        new_value = min + rel * range;
        changed = true;
    }

    // Persist state
    STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.insert(
            id,
            (dragging, drag_offset),
        );
    });

    (new_value, changed)
}

/// A simple dropdown that shows `options` when the button is pressed.
/// Returns `Some(selected)` when the user picks a different entry,
/// otherwise `None`.
pub fn gui_dropdown<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
) -> Option<T> {
    // Button
    let button_clicked = gui_button(rect, label);

    // Load previous state
    let mut state = dropdown_state::get(id);

    // Decide whether the list should be open this frame
    let list_is_open = button_clicked || state.open;
    state.open = list_is_open; // Remember for next frame   

    // Let the editor know a dropdown is open
    let mut any_open = false;
    DROPDOWN_OPEN.with(|f| {
        let was = *f.borrow();
        *f.borrow_mut() = was || list_is_open;
        any_open = *f.borrow();
    });     

    // Compute the list rectangle
    let list_rect = Rect::new(
        rect.x,
        rect.y + rect.h,
        rect.w,
        rect.h * options.len() as f32,
    );

    if list_is_open {
        state.rect = list_rect;             
    }

    // Draw the list and handle selection
    if list_is_open {
        // Background
        draw_rectangle(
            list_rect.x,
            list_rect.y,
            list_rect.w,
            list_rect.h,
            Color::new(0., 0., 0., 1.0),
        );

        let mouse_pos = mouse_position().into();
        for (i, opt) in options.iter().enumerate() {
            let entry_rect = Rect::new(
                list_rect.x,
                list_rect.y + i as f32 * rect.h,
                list_rect.w,
                rect.h,
            );

            let hovered = entry_rect.contains(mouse_pos);
            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                // Close the list and return the chosen value
                state.open = false;
                dropdown_state::set(id, state);
                update_global_dropdown_flag();
                return Some(opt.clone());
            }

            if hovered {
                draw_rectangle(
                    entry_rect.x,
                    entry_rect.y,
                    entry_rect.w,
                    entry_rect.h,
                    Color::new(0.2, 0.2, 0.2, 0.9),
                );
            }

            draw_text(
                &to_string(opt),
                entry_rect.x + 5.,
                entry_rect.y + entry_rect.h * 0.7,
                20.,
                WHITE,
            );

            // Draw the outline last
            draw_rectangle_lines(
                list_rect.x, 
                list_rect.y, 
                list_rect.w, 
                list_rect.h, 
                2., 
                WHITE
            );
        }
    }

    // Clicking outside closes the dropdown
    let mouse_pos = mouse_position().into();
    if is_mouse_button_pressed(MouseButton::Left)
        && !rect.contains(mouse_pos)
        && !(state.open && state.rect.contains(mouse_pos))
    {
        state.open = false;
    }

    // Persist the state
    dropdown_state::set(id, state);
    update_global_dropdown_flag();
    None
}

/// Helper module that stores the temporary dropdown state.
mod dropdown_state {
    use macroquad::prelude::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::ui::widgets::WidgetId;

    thread_local! {
        pub static STATE: RefCell<HashMap<WidgetId, DropState>> =
            RefCell::new(HashMap::new());
    }

    #[derive(Clone, Copy)]
    pub struct DropState {
        pub open: bool,
        pub rect: Rect,
    }

    impl Default for DropState {
        fn default() -> Self {
            Self { open: false, rect: Rect::default() }
        }
    }

    pub fn get(key: WidgetId) -> DropState {
        STATE.with(|s| {
            *s.borrow()
                .get(&key)
                .unwrap_or(&DropState::default())
        })
    }

    pub fn set(key: WidgetId, value: DropState) {
        STATE.with(|s| {
            s.borrow_mut().insert(key, value);
        });
    }
}

// helper, called at the end of gui_dropdown
fn update_global_dropdown_flag() {
    dropdown_state::STATE.with(|s| {
        let any = s.borrow().values().any(|st| st.open);
        DROPDOWN_OPEN.with(|f| *f.borrow_mut() = any);
    });
}

/// A simple toast that disappears after a short delay.
pub struct WarningToast {
    /// Text that will be shown.
    pub msg: String,
    /// When the toast was created.
    start: Instant,
    /// How long the toast stays visible (seconds).
    pub duration: f32,
    /// Whether the toast is currently visible.
    pub active: bool,
}

impl WarningToast {
    /// Create a new toast that lives for `duration` seconds.
    pub fn new<S: Into<String>>(msg: S, duration: f32) -> Self {
        Self {
            msg: msg.into(),
            start: Instant::now(),
            duration,
            active: true,
        }
    }

    /// Call each frame. Draws the toast if it is still alive.
    pub fn update(&mut self) {
        if !self.active {
            return;
        }
        // Hide after the elapsed time.
        if self.start.elapsed().as_secs_f32() >= self.duration {
            self.active = false;
            return;
        }
        
        let txt = measure_text(&self.msg, None, 18, 1.0);

        // Top left
        let bg_rect = Rect::new(
            WIDGET_PADDING,                         
            WIDGET_PADDING,                        
            txt.width + WIDGET_PADDING * 2.0,       
            txt.height + WIDGET_PADDING * 2.0,      
        );

        // Background
        draw_rectangle(
            bg_rect.x,
            bg_rect.y,
            bg_rect.w,
            bg_rect.h,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        // Text
        draw_text(
            &self.msg,
            bg_rect.x + WIDGET_PADDING,
            bg_rect.y + txt.height + WIDGET_PADDING / 2.0,
            18.0,
            WHITE,
        );
    }
}