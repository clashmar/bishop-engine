//! Backend text rendering functions.

#[cfg(feature = "macroquad")]
mod macroquad_text {
    use crate::text::{TextDimensions, TextParams};
    use crate::types::Color;
    use macroquad::prelude as mq;
    use std::cell::RefCell;

    thread_local! {
        static FONT: RefCell<Option<mq::Font>> = RefCell::new(None);
    }

    /// Sets the font to use for text rendering.
    pub fn set_font(font: mq::Font) {
        FONT.with(|f| {
            *f.borrow_mut() = Some(font);
        });
    }

    /// Draws text at the specified position and returns its dimensions.
    pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
        FONT.with(|f| {
            let font_ref = f.borrow();
            let font = font_ref.as_ref();
            let dims = mq::measure_text(text, font, font_size as u16, 1.0);
            mq::draw_text_ex(
                text,
                x,
                y,
                mq::TextParams {
                    font,
                    font_size: font_size as u16,
                    color: color.into(),
                    ..Default::default()
                },
            );
            TextDimensions {
                width: dims.width,
                height: dims.height,
                offset_y: dims.offset_y,
            }
        })
    }

    /// Draws text with extended parameters.
    pub fn draw_text_ex(text: &str, x: f32, y: f32, params: TextParams) -> TextDimensions {
        FONT.with(|f| {
            let font_ref = f.borrow();
            let font = font_ref.as_ref();

            let dims = mq::measure_text(text, font, params.font_size, params.font_scale);

            let params = mq::TextParams {
                font_size: params.font_size as u16,
                color: mq::BLACK,
                rotation: params.rotation,
                font: params.font,
                ..Default::default()
            };

            mq::draw_text_ex(text, x, y, params);

            TextDimensions {
                width: dims.width,
                height: dims.height,
                offset_y: dims.offset_y,
            }
        })
    }

    /// Measures text without drawing it.
    pub fn measure_text(text: &str, font_size: f32) -> TextDimensions {
        FONT.with(|f| {
            let font_ref = f.borrow();
            let font = font_ref.as_ref();
            let dims = mq::measure_text(text, font, font_size as u16, 1.0);
            TextDimensions {
                width: dims.width,
                height: dims.height,
                offset_y: dims.offset_y,
            }
        })
    }

    /// Initializes the backend with the GNF font.
    pub fn init_with_gnf() {
        crate::text::font::precache();
        if let Some(font) = crate::text::font::get_font() {
            set_font(font);
        }
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_text::*;
