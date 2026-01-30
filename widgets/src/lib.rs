mod widget_id;
mod constants;
mod state;
mod helpers;
mod clipboard;
mod widgets;

pub use widget_id::*;
pub use constants::*;
pub use state::*;
pub use helpers::*;
pub use widgets::*;

use macroquad::prelude::*;
use std::cell::RefCell;

pub trait TextRenderer: Send + Sync {
    fn draw_text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions;
    fn measure_text(&self, text: &str, font_size: f32, font_scale: f32) -> TextDimensions;
}

struct DefaultTextRenderer;

impl TextRenderer for DefaultTextRenderer {
    fn draw_text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font_size: font_size as u16,
                color,
                ..Default::default()
            },
        )
    }

    fn measure_text(&self, text: &str, font_size: f32, font_scale: f32) -> TextDimensions {
        measure_text(text, None, font_size as u16, font_scale)
    }
}

thread_local! {
    static TEXT_RENDERER: RefCell<&'static dyn TextRenderer> = RefCell::new(&DefaultTextRenderer);
}

pub fn set_text_renderer(renderer: &'static dyn TextRenderer) {
    TEXT_RENDERER.with(|r| {
        *r.borrow_mut() = renderer;
    });
}

pub(crate) fn draw_text_ui(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    TEXT_RENDERER.with(|r| {
        r.borrow().draw_text(text, x, y, font_size, color)
    })
}

pub(crate) fn measure_text_ui(text: &str, font_size: f32, font_scale: f32) -> TextDimensions {
    TEXT_RENDERER.with(|r| {
        r.borrow().measure_text(text, font_size, font_scale)
    })
}
