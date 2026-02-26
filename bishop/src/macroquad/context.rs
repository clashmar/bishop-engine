//! Macroquad context struct.

/// Macroquad backend implementation wrapping global functions.
pub struct MacroquadContext {
    pub(crate) char_buffer: Vec<char>,
    pub(crate) fullscreen: bool,
}

impl MacroquadContext {
    /// Creates a new macroquad context.
    pub fn new() -> Self {
        Self {
            char_buffer: Vec::new(),
            fullscreen: false,
        }
    }
}

impl Default for MacroquadContext {
    fn default() -> Self {
        Self::new()
    }
}
