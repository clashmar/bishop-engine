//! Macroquad context struct.

use macroquad::prelude as mq;

/// Macroquad backend implementation wrapping global functions.
pub struct MacroquadContext {
    pub(crate) char_buffer: Vec<char>,
}

impl MacroquadContext {
    /// Creates a new macroquad context.
    pub fn new() -> Self {
        Self {
            char_buffer: Vec::new(),
        }
    }

    /// Updates the character buffer. Call once per frame before processing input.
    pub fn update(&mut self) {
        self.char_buffer.clear();
        while let Some(c) = mq::get_char_pressed() {
            self.char_buffer.push(c);
        }
    }
}

impl Default for MacroquadContext {
    fn default() -> Self {
        Self::new()
    }
}
