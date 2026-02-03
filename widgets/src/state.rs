use std::collections::HashMap;
use std::cell::RefCell;
use crate::*;

/// Keys that support hold-to-repeat behavior.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepeatableKey {
    Backspace,
    Delete,
    Left,
    Right,
}

/// State for a text input widget.
pub struct TextInputState {
    pub text: String,
    pub cursor_char: usize,
    pub focused: bool,
    pub selection_anchor: Option<usize>,
    pub last_key_time: f64,
    pub repeat_key: Option<RepeatableKey>,
    pub repeat_started: bool,
    pub dragging: bool,
    pub scroll_offset_x: f32,
}

impl TextInputState {
    pub fn new(text: String) -> Self {
        let cursor_char = text.chars().count();
        Self {
            text,
            cursor_char,
            focused: false,
            selection_anchor: None,
            last_key_time: 0.0,
            repeat_key: None,
            repeat_started: false,
            dragging: false,
            scroll_offset_x: 0.0,
        }
    }
}

/// State for a number input widget.
pub struct NumberInputState {
    pub text: String,
    pub cursor_char: usize,
    pub focused: bool,
    pub selection_anchor: Option<usize>,
    pub last_key_time: f64,
    pub repeat_key: Option<RepeatableKey>,
    pub repeat_started: bool,
    pub dragging: bool,
    pub scroll_offset_x: f32,
}

impl NumberInputState {
    pub fn new(text: String) -> Self {
        let cursor_char = text.chars().count();
        Self {
            text,
            cursor_char,
            focused: false,
            selection_anchor: None,
            last_key_time: 0.0,
            repeat_key: None,
            repeat_started: false,
            dragging: false,
            scroll_offset_x: 0.0,
        }
    }
}

thread_local! {
    pub static INPUT_TEXT_STATE: RefCell<HashMap<WidgetId, TextInputState>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    pub static INPUT_NUMBER_STATE: RefCell<HashMap<WidgetId, NumberInputState>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    pub static DROPDOWN_OPEN: RefCell<bool> = const { RefCell::new(false) };
}

pub fn is_dropdown_open() -> bool {
    DROPDOWN_OPEN.with(|f| *f.borrow())
}

pub fn widgets_frame_start() {
    tab_registry_clear();
}

pub fn widgets_frame_end() {
    resolve_pending_tab();
}