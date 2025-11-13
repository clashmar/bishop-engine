// engine_core/src/ui/toast.rs
use std::time::Instant;
use macroquad::prelude::*;

use crate::ui::{text::{draw_text_ui, measure_text_ui}, widgets::DEFAULT_FONT_SIZE};

const PADDING: f32 = 20.0;

/// A simple toast that disappears after a short delay.
pub struct Toast {
    /// Text that will be shown.
    pub msg: String,
    /// When the toast was created.
    start: Instant,
    /// How long the toast stays visible (seconds).
    pub duration: f32,
    /// Whether the toast is currently visible.
    pub active: bool,
}

impl Toast {
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
        
        let txt = measure_text_ui(&self.msg, DEFAULT_FONT_SIZE, 1.0);

        // Top left
        let bg_rect = Rect::new(
            PADDING,                                          
            screen_height() - PADDING - (txt.height + PADDING),
            txt.width + PADDING * 2.0,
            txt.height + PADDING,
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
        draw_text_ui(
            &self.msg,
            bg_rect.x + PADDING,
            bg_rect.y + txt.height + PADDING / 2.0,
            DEFAULT_FONT_SIZE,
            WHITE,
        );
    }
}