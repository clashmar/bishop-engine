mod widget_id;
mod constants;
mod state;
mod helpers;
mod clipboard;
mod widgets;
mod tab_registry;
mod focus;

pub use widget_id::*;
pub use constants::*;
pub use state::*;
pub use helpers::*;
pub use widgets::*;
pub use tab_registry::*;
pub use focus::*;

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

/// Draws text within a clipped rectangle with horizontal scroll offset.
pub(crate) fn draw_text_clipped(
    text: &str,
    rect_x: f32,
    rect_y: f32,
    rect_w: f32,
    rect_h: f32,
    scroll_offset: f32,
    font_size: f32,
    color: Color,
) {
    let text_x = rect_x + WIDGET_PADDING / 2. - scroll_offset;
    let text_y = rect_y + rect_h * 0.7;

    let clip_left = rect_x + WIDGET_PADDING / 2.;
    let clip_right = rect_x + rect_w - WIDGET_PADDING / 2.;

    let mut visible_start_byte = 0;
    let mut visible_end_byte = text.len();
    let mut render_x = text_x;
    let mut found_start = false;

    for (byte_idx, ch) in text.char_indices() {
        let char_start_x = text_x + measure_text_ui(&text[..byte_idx], font_size, 1.0).width;
        let char_end_x = text_x + measure_text_ui(&text[..byte_idx + ch.len_utf8()], font_size, 1.0).width;

        if !found_start && char_start_x >= clip_left {
            visible_start_byte = byte_idx;
            render_x = char_start_x;
            found_start = true;
        }

        if char_end_x > clip_right {
            visible_end_byte = byte_idx;
            break;
        }
    }

    if visible_start_byte < visible_end_byte {
        let visible_text = &text[visible_start_byte..visible_end_byte];
        draw_text_ui(visible_text, render_x, text_y, font_size, color);
    }
}
