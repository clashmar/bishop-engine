// engine_core/src/ui/widgets.rs
use crate::*;
use crate::script::script::ScriptId;
use crate::script::script_manager::ScriptManager;
use std::borrow::Cow;
use crate::assets::asset_manager::AssetManager;
use crate::assets::sprite::SpriteId;
use crate::ui::text::*;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::Display;
use std::str::FromStr;
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

pub const WIDGET_PADDING: f32 = 10.0; 
pub const WIDGET_SPACING: f32 = 10.0;   
pub const DEFAULT_FONT_SIZE_16: f32 = 16.0;
pub const HEADER_FONT_SIZE_20: f32 = 20.0;
pub const FIELD_TEXT_SIZE_16: f32 = 16.0; 
pub const FIELD_TEXT_COLOR: Color = WHITE;
pub const DEFAULT_FIELD_HEIGHT: f32 = 30.0;
pub const DEFAULT_CHECKBOX_DIMS: f32 = 20.0;

// Colours
pub const OUTLINE_COLOR: Color = WHITE;
pub const FIELD_BACKGROUND_COLOR: Color = Color::new(0., 0., 0., 1.0);

const HOLD_INITIAL_DELAY: f64 = 0.50;
const HOLD_REPEAT_RATE: f64 = 0.05;
const PLACEHOLDER_TEXT: &'static str = "<type here>";  

thread_local! {
    static INPUT_TEXT_STATE: RefCell<HashMap<WidgetId, (String, usize, bool, f64, bool)>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    static INPUT_NUMBER_STATE: RefCell<HashMap<WidgetId, (String, usize, bool)>> =
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
    pub static DROPDOWN_OPEN: RefCell<bool> = RefCell::new(false);
}

/// Global flag that tells the rest of the editor whether a dropdown
/// is currently open.
pub fn is_dropdown_open() -> bool {
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

/// Same as `gui_input_text_default` but clamps the text to `max_len`.
pub fn gui_input_text_clamped(id: WidgetId, rect: Rect, current: &str, max_len: usize) -> (String, bool) {
    gui_input_text(id, rect, current, false, Some(max_len))
}

/// Same as `gui_input_text_focused` but clamps the text to `max_len`.
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
    let mut just_gained_focus = false;

    // Load / initialise widget state
    let mut text = current.to_string();
    let mut cursor_char = 0usize;
    let mut focused = false;
    let mut last_backspace = 0.0_f64;
    let mut repeat_started = false;

    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();

        if let Some((saved_text, saved_cur, saved_foc, saved_time, saved_repeat)) = map.get(&id) {
            text = saved_text.clone();
            focused = if start_focused { true } else { *saved_foc };
            just_gained_focus = start_focused && !*saved_foc;
            cursor_char = if start_focused && just_gained_focus { text.chars().count() } else { *saved_cur };
            last_backspace = *saved_time;
            repeat_started = *saved_repeat;
        } else {
            focused = start_focused;
            just_gained_focus = start_focused;
            cursor_char = text.chars().count();
            map.insert(id, (text.clone(), cursor_char, focused, last_backspace, repeat_started));
        }
    });

    // Stop the widget overwriting the next component
    if !focused {
        if text != current {
            text = current.to_string();
            cursor_char = text.len();
        }
    }

    // Draw background & current text
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD_BACKGROUND_COLOR);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);
    let display = if text.is_empty() { PLACEHOLDER_TEXT } else { &text };

    draw_input_field_text(display, rect);

    // Focus handling
    let mouse = mouse_position();
    let mouse_over = rect.contains(vec2(mouse.0, mouse.1));
    if is_mouse_button_pressed(MouseButton::Left) {
        // Clicking inside gains focus, clicking elsewhere loses focus
        if !focused && mouse_over {
            just_gained_focus = true;
        }
        focused = mouse_over;

        if !focused {
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
        }
    }

    if just_gained_focus {
        // Discard everything that was typed while the widget was not active
        while let Some(_) = get_char_pressed() {}
    }

    // Don't update the field if a dropdown is open
    if is_dropdown_open() {
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
        
        // Delete
        if is_key_pressed(KeyCode::Delete) && cursor_char < text.chars().count() {
            let start = byte_offset(&text, cursor_char);
            let end = byte_offset(&text, cursor_char + 1);
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
            if chr.is_ascii_graphic() || chr == ' ' {
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
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
            focused = false;
        }
    }

    // Blinking cursor
    let now = get_time();
    if focused && ((now * 2.0) as i32 % 2 == 0) {
        let byte_pos = byte_offset(&text, cursor_char);
        let prefix = &text[..byte_pos];
        let cursor_x = rect.x + 5. + measure_text_ui(prefix, DEFAULT_FONT_SIZE_16, 1.0).width;
        draw_line(
            cursor_x,
            rect.y + rect.h * 0.3,
            cursor_x,
            rect.y + rect.h * 0.8,
            2.,
            OUTLINE_COLOR,
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

pub fn gui_input_number_i32(id: WidgetId, rect: Rect, current: i32) -> i32 {
    gui_input_number_generic(id, rect, current)
}

pub fn gui_input_number_f32(id: WidgetId, rect: Rect, current: f32) -> f32 {
    gui_input_number_generic(id, rect, current)
}

/// Generic numeric widget.
pub fn gui_input_number_generic<T>(
    id: WidgetId,
    rect: Rect,
    current: T,
) -> T
where
    T: FromStr + Display + Default + Copy + PartialEq,
    <T as FromStr>::Err: std::fmt::Debug,
{
    // Load or initialise the entry for this widget
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

    // Stop the widget overwriting the next component
    if !focused {
        if text.parse::<T>().unwrap_or_default() != current {
            text = current.to_string();
            cursor_char = text.len();
        }
    }

    // Draw background & current text
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD_BACKGROUND_COLOR);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., WHITE);
    let placeholder = "<#>";
    let display = if text.is_empty() { placeholder } else { &text };

    draw_input_field_text(display, rect);

    // Abort input handling if a dropdown blocks interaction
    if is_dropdown_open() {
        return current;
    }

    let mouse = mouse_position();
    let mouse_over = rect.contains(vec2(mouse.0, mouse.1));
    if is_mouse_button_pressed(MouseButton::Left) {
        focused = mouse_over;
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
            // Tell the rest of the editor that a text field is active
            INPUT_FOCUSED.with(|f| *f.borrow_mut() = true);

            if chr.is_control() {
                continue;
            }

            // Allow a leading minus sign only for types that can represent negatives
            if chr == '-' && cursor_char == 0 && !text.starts_with('-') && T::from_str("-0").is_ok() {
                text.insert(cursor_char, chr);
                cursor_char += 1;
                continue;
            }

            // Floating point numbers
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

    // Persist state for the next frame
    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.insert(id, (text.clone(), cursor_char, focused));
    });

    // Return the parsed value
    text.parse::<T>().unwrap_or(current)
}

/// Remove any stored state for the given numeric widget.
pub fn gui_input_number_reset(id: WidgetId) {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.remove(&id);
    });
}

/// Clears the focused flag of all text fields.
pub fn clear_all_input_focus() {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();
        for (_, entry) in map.iter_mut() {
            entry.2 = false;
        }
    });
    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        for (_, entry) in map.iter_mut() {
            entry.2 = false;
        }
    });
}

/// Simple toggle widget. Returns `true` when the value changed this frame.
pub fn gui_checkbox(rect: Rect, value: &mut bool) -> bool {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD_BACKGROUND_COLOR);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., OUTLINE_COLOR);

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
    if is_dropdown_open() {
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

/// Possible styles for a button.
pub enum ButtonStyle {
    Default,
    Plain,
}

/// Rectangular button with background and outline. Returns `true` on click.
pub fn gui_button(rect: Rect, label: &str) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Default, FIELD_TEXT_COLOR, Vec2::ZERO)
}

/// Rectangular button with no background or outline. Returns `true` on click.
pub fn gui_button_plain(rect: Rect, label: &str, text_color: Color) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Plain, text_color, Vec2::ZERO)
}

/// Default button with text offset. Returns `true` on click.
pub fn gui_button_y_offset(rect: Rect, label: &str, text_offset: Vec2) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Default, FIELD_TEXT_COLOR, text_offset)
}

fn gui_button_impl(
    rect: Rect, 
    label: &str, 
    style: ButtonStyle, 
    text_color: Color,
    text_offset: Vec2,
) -> bool {
    let mouse = mouse_position();
    let mut hovered = rect.contains(vec2(mouse.0, mouse.1));

    // Common text layout
    let txt_dims = measure_text_ui(label, FIELD_TEXT_SIZE_16, 1.0);
    let txt_y = rect.y + rect.h * 0.7;
    let mut txt_x = rect.x;

    match style {
        ButtonStyle::Default => {
            // Background, Outline & Hover
            let hovered = rect.contains(vec2(mouse.0, mouse.1));
            let background = if hovered && !is_dropdown_open() {
                Color::new(0.2, 0.2, 0.2, 0.8)
            } else {
                FIELD_BACKGROUND_COLOR
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., OUTLINE_COLOR);
            txt_x = rect.x + (rect.w - txt_dims.width) / 2.;
        }
        ButtonStyle::Plain => {
            // Hover only
            let width = txt_dims.width + WIDGET_PADDING * 2.0;
            txt_x = txt_x + WIDGET_PADDING;

            hovered = Rect::new(rect.x, rect.y, width, rect.h)
                .contains(vec2(mouse.0, mouse.1));

            if hovered && !is_dropdown_open() {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    width,
                    rect.h,
                    Color::new(0.0, 0.0, 0.0, 0.5),
                );
            }
        }
    }
    
    draw_text_ui(label, txt_x + text_offset.x, txt_y + text_offset.y, FIELD_TEXT_SIZE_16, text_color);

    is_mouse_button_pressed(MouseButton::Left) 
    && hovered 
    && !is_dropdown_open()
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
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD_BACKGROUND_COLOR);
    draw_rectangle(rect.x, track_y, rect.w, track_h, Color::new(0.2, 0.2, 0.2, 0.8));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., OUTLINE_COLOR);

    let handle_col = if dragging && !is_dropdown_open() {
        Color::new(0.6, 0.6, 0.9, 1.0)
    } else {
        Color::new(0.4, 0.4, 0.8, 1.0)
    };
    draw_rectangle(handle_x, rect.y, handle_sz, rect.h, handle_col);
    draw_rectangle_lines(handle_x, rect.y, handle_sz, rect.h, 2., WHITE);

    if is_dropdown_open() {
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

/// Possible styles for a dropdown menu.
pub enum DropDownStyle {
    /// Uses default button to open dropdown.
    Default,
    /// Uses plain button to open dropdown.
    Plain,
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
    gui_dropdown_impl(
        id, 
        rect, 
        label, 
        options, 
        to_string, 
        DropDownStyle::Default, 
        WHITE,
        0.0,
    )
}

/// Same as gui_dropdown but uses a plain button. Text color sets the color 
/// of the button text and y_offset moves the options.
pub fn gui_dropdown_plain<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
    text_color: Color,
    y_offset: f32,
) -> Option<T> {
    gui_dropdown_impl(
        id, 
        rect, 
        label, 
        options, 
        to_string, 
        DropDownStyle::Plain, 
        text_color,
        y_offset
    )
}

fn gui_dropdown_impl<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
    style: DropDownStyle,
    text_color: Color,
    y_offset: f32,
) -> Option<T> {
    const MAX_VISIBLE_ROWS: usize = 8;
    const SCROLL_SPEED: f32 = 5.0;
    const W_PADDING: f32 = 8.0;
    const SCROLLBAR_WIDTH: f32 = 6.0;

    // Load previous state
    let mut state = dropdown_state::get(id);

    // Temporarily set dropdown open to false so the button is still interactable
    let prev_state = state.open;
    state.open = false;
    dropdown_state::set(id, state);
    update_global_dropdown_flag();

    let button_clicked = match style {
        DropDownStyle::Default => {
            gui_button(rect, label)
        }
        DropDownStyle::Plain => {
            gui_button_plain(rect, label, text_color)
        }
    };

    // Set it back to the previous state
    state.open = prev_state;
    dropdown_state::set(id, state);
    update_global_dropdown_flag();

    if button_clicked {
        state.open = !state.open;
    }

    // Decide whether the list should be open this frame
    let list_is_open = state.open; 
    state.open = list_is_open; // Remember for next frame   

    // Let the editor know a dropdown is open
    let mut any_open = false;
    DROPDOWN_OPEN.with(|r| {
        let was = *r.borrow();
        *r.borrow_mut() = was || list_is_open;
        any_open = *r.borrow();
    });     

    // Compute the widest option
    let mut max_opt_width = 0.0_f32;
    for opt in options.iter() {
        let txt = to_string(opt);
        let width = measure_text_ui(&txt, DEFAULT_FONT_SIZE_16, 1.0).width;
        if width > max_opt_width {
            max_opt_width = width;
        }
    }

    let list_width = rect.w
        .max(max_opt_width + 2.0 * W_PADDING + SCROLLBAR_WIDTH);

    // Compute the list rectangle
    let visible_rows = MAX_VISIBLE_ROWS.min(options.len());
    let list_rect = Rect::new(
        rect.x,
        rect.y + rect.h + y_offset,
        list_width,
        rect.h * visible_rows as f32,
    );

    if list_is_open {
        state.rect = list_rect;             
    }

    // Draw the list and handle selection
    if list_is_open {
        let total_height = rect.h * options.len() as f32;
        let max_offset = (total_height - list_rect.h).max(0.0);

        let mouse_pos = mouse_position().into();

        if list_rect.contains(mouse_pos) {
            let (_, wheel_y) = mouse_wheel();
            if wheel_y != 0.0 {
                let delta = wheel_y * SCROLL_SPEED;
                state.scroll_offset = (state.scroll_offset - delta)
                    .clamp(0.0, max_offset);
            }
        }

        // Background
        draw_rectangle(
            list_rect.x,
            list_rect.y,
            list_rect.w,
            list_rect.h,
            FIELD_BACKGROUND_COLOR,
        );
        
        for (i, opt) in options.iter().enumerate() {
            // The Y position the entry would have without scrolling
            let entry_y = list_rect.y + i as f32 * rect.h;

            // Apply the scroll offset
            let draw_y = entry_y - state.scroll_offset;

            // Skip entries that are above or below the visible area
            if draw_y + rect.h < list_rect.y + rect.h          
                || draw_y > list_rect.y + list_rect.h - rect.h
            {
                continue;
            }

            let entry_rect = Rect::new(
                list_rect.x,
                draw_y,
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
            
            draw_text_ui(
                &to_string(opt),
                entry_rect.x + 5.,
                entry_rect.y + entry_rect.h * 0.7,
                DEFAULT_FONT_SIZE_16,
                FIELD_TEXT_COLOR,
            );

            // Scrollbar on the right hand side
            let total_height = rect.h * options.len() as f32;
            if total_height > list_rect.h {
                // Proportion of visible area
                let thumb_h = (list_rect.h / total_height) * list_rect.h;
                // Position of the thumb
                let thumb_y = list_rect.y + (state.scroll_offset / (total_height - list_rect.h)) * (list_rect.h - thumb_h);

                // Background track
                draw_rectangle(
                    list_rect.x + list_rect.w - 6.,
                    list_rect.y,
                    6.,
                    list_rect.h,
                    Color::new(0.2, 0.2, 0.2, 0.5),
                );
                // Thumb
                draw_rectangle(
                    list_rect.x + list_rect.w - 6.,
                    thumb_y,
                    6.,
                    thumb_h,
                    Color::new(0.6, 0.6, 0.6, 0.9),
                );
            }

            // Draw the outline last
            draw_rectangle_lines(
                list_rect.x, 
                list_rect.y, 
                list_rect.w, 
                list_rect.h, 
                2., 
                OUTLINE_COLOR
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
pub mod dropdown_state {
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
        pub scroll_offset: f32,
    }

    impl Default for DropState {
        fn default() -> Self {
            Self { 
                open: false, 
                rect: Rect::default(),
                scroll_offset: 0.,
            }
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

// Helper, called at the end of gui_dropdown
pub fn update_global_dropdown_flag() {
    dropdown_state::STATE.with(|s| {
        let any = s.borrow().values().any(|st| st.open);
        DROPDOWN_OPEN.with(|f| *f.borrow_mut() = any);
    });
}

pub fn gui_stepper(
    rect: Rect,
    label: &str,
    steps: &[f32],
    current: f32,
) -> f32 {
    let mut idx = steps
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (*a - current).abs()
                .partial_cmp(&(*b - current).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Layout
    const Y_OFFSET: f32 = 15.0;

    let label = format!("{}:", label);
    let label_width = measure_text_ui(&label, FIELD_TEXT_SIZE_16, 1.0).width;

    let btn_w = FIELD_TEXT_SIZE_16 * 1.2;
    let val_w = measure_text_ui("3.0", FIELD_TEXT_SIZE_16, 1.0).width + WIDGET_SPACING + 5.0;

    // Label
    draw_text_ui(&label, rect.x, rect.y, FIELD_TEXT_SIZE_16, FIELD_TEXT_COLOR);

    // Display value
    let val_rect = Rect::new(
        rect.x + label_width + WIDGET_SPACING,
        rect.y - Y_OFFSET,
        val_w,
        rect.h,
    );

    // White outline
    draw_rectangle_lines(
        val_rect.x,
        val_rect.y - 7.5,
        val_rect.w,
        btn_w + 15.0,
        2.,
        OUTLINE_COLOR,
    );

    let txt = format!("{:.1}", steps[idx]);
    draw_text_ui(
        &txt,
        val_rect.x + 7.5,
        val_rect.y + 17.5,
        FIELD_TEXT_SIZE_16,
        FIELD_TEXT_COLOR,
    );

    // “‑” button
    let decrease_rect = Rect::new(
        val_rect.x + val_w + WIDGET_SPACING,
        rect.y - Y_OFFSET,
        btn_w,
        btn_w,
    );

    if gui_button(decrease_rect, "-") && idx > 0 {
        idx -= 1;
    }

    // “+” button
    let increase_rect = Rect::new(
        decrease_rect.x + btn_w + WIDGET_SPACING,
        rect.y - Y_OFFSET,
        btn_w,
        btn_w,
    );
    if gui_button(increase_rect, "+") && idx + 1 < steps.len() {
        idx += 1;
    }

    steps[idx]
}

/// UI widget that can choose a PNG from disk to update a SriteId, or remove it.
/// Returns true if the sprite was updated.
pub fn gui_sprite_picker(
    rect: Rect,
    id: &mut SpriteId,
    asset_manager: &mut AssetManager,
) -> bool {
    let btn_label: Cow<str> = if id.0 == 0 {
        Cow::Borrowed("[Pick File]")
    } else {
        let filename = asset_manager
            .sprite_id_to_path
            .get(id)
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "???".to_string());

        Cow::Owned(format!("[/{}]", filename))
    };

    let remove_w = rect.h; // square button
    let picker_w = rect.w - remove_w - WIDGET_SPACING;

    let picker_rect = Rect::new(rect.x, rect.y, picker_w, rect.h);
    let remove_rect = Rect::new(
        rect.x + rect.w - remove_w,
        rect.y,
        remove_w,
        rect.h,
    );

    let mut changed = false;

    if gui_button(picker_rect, &btn_label) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG images", &["png"])
                .pick_file()
            {
                let normalized = asset_manager.normalize_path(path);
                match asset_manager.get_or_load(&normalized) {
                    Some(new_id) => {
                        *id = new_id;
                        changed = true;
                    }
                    None => {
                        onscreen_error!("Failed to load sprite.");
                    }
                }
            }
        }
    }

    if gui_button(remove_rect, "x") && id.0 != 0 {
        *id = SpriteId(0);
        changed = true;
    }

    changed
}

/// UI widget that can choose a PNG from disk to update a SriteId, or remove it.
/// Returns true if the sprite was updated.
pub fn gui_script_picker(
    rect: Rect,
    id: &mut ScriptId,
    script_manager: &mut ScriptManager,
) -> bool {
    let btn_label: Cow<str> = if id.0 == 0 {
        Cow::Borrowed("[Pick File]")
    } else {
        let filename = script_manager
            .script_id_to_path
            .get(id)
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "???".to_string());

        Cow::Owned(format!("[/{}]", filename))
    };

    let remove_w = rect.h; // square button
    let picker_w = rect.w - remove_w - WIDGET_SPACING;

    let picker_rect = Rect::new(rect.x, rect.y, picker_w, rect.h);
    let remove_rect = Rect::new(
        rect.x + rect.w - remove_w,
        rect.y,
        remove_w,
        rect.h,
    );

    let mut changed = false;

    if gui_button(picker_rect, &btn_label) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Lua Scripts", &["lua"])
                .pick_file()
            {
                let normalized = script_manager.normalize_path(path);
                match script_manager.get_or_load(&normalized) {
                    Some(new_id) => {
                        *id = new_id;
                        changed = true;
                    }
                    None => {
                        onscreen_error!("Failed to load script.");
                    }
                }
            }
        }
    }

    if gui_button(remove_rect, "x") && id.0 != 0 {
        *id = ScriptId(0);
        changed = true;
    }

    changed
}

/// Draws the text for an input widget. Can be called by non-widgets.
pub fn draw_input_field_text(text: &str, rect: Rect) {
    draw_text_ui(
        text,
        rect.x + WIDGET_PADDING / 2.,
        rect.y + rect.h * 0.7,
        DEFAULT_FONT_SIZE_16,
        FIELD_TEXT_COLOR,
    );
}

/// Returns the x position and width for text to be centered around a given x position.
pub fn center_text_field(x: f32, text: &str) -> (f32, f32) {
    let text_to_measure = if text.is_empty() { PLACEHOLDER_TEXT } else { text };
    let text_size = measure_text_ui(text_to_measure, DEFAULT_FONT_SIZE_16, 1.0);
    let new_x = x - (text_size.width / 2.);
    (new_x - WIDGET_PADDING / 2., text_size.width + WIDGET_PADDING)
}

/// Returns the x position and width for text to be centered around a given x position.
pub fn rect_width_for_text(text: &str, font_size: f32) -> f32 {
    measure_text_ui(text, font_size, 1.0).width + WIDGET_PADDING * 2.0
}


