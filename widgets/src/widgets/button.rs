use macroquad::prelude::*;
use crate::{
    draw_text_ui, measure_text_ui, is_dropdown_open,
    FIELD_BACKGROUND_COLOR, OUTLINE_COLOR, FIELD_TEXT_COLOR, FIELD_TEXT_SIZE_16,
    HOVER_COLOR, HOVER_COLOR_PLAIN,
};

/// The visual style of a button.
#[derive(Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    /// Standard button with background and border.
    Default,
    /// Minimal button with no background, only shows hover state.
    Plain,
}

/// A clickable button widget using the builder pattern.
pub struct Button<'a> {
    rect: Rect,
    label: &'a str,
    style: ButtonStyle,
    text_color: Color,
    hover_color: Color,
    text_offset: Vec2,
    blocked: bool,
}

impl<'a> Button<'a> {
    /// Creates a new button with the given rect and label.
    pub fn new(rect: Rect, label: &'a str) -> Self {
        Self {
            rect,
            label,
            style: ButtonStyle::Default,
            text_color: FIELD_TEXT_COLOR,
            hover_color: HOVER_COLOR,
            text_offset: Vec2::ZERO,
            blocked: false,
        }
    }

    /// Sets the button to use the plain style (no background).
    pub fn plain(mut self) -> Self {
        self.style = ButtonStyle::Plain;
        self.hover_color = HOVER_COLOR_PLAIN;
        self
    }

    /// Sets the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Sets the hover background color.
    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    /// Sets an offset for the text position.
    pub fn text_offset(mut self, offset: Vec2) -> Self {
        self.text_offset = offset;
        self
    }

    /// Sets whether the button is blocked from interaction.
    pub fn blocked(mut self, blocked: bool) -> Self {
        self.blocked = blocked;
        self
    }

    /// Draws the button and returns true if clicked.
    pub fn show(self) -> bool {
        let mouse = mouse_position();
        let hovered = self.rect.contains(vec2(mouse.0, mouse.1));

        let txt_dims = measure_text_ui(self.label, FIELD_TEXT_SIZE_16, 1.0);
        let txt_y = self.rect.y + self.rect.h * 0.7;
        let txt_x = self.rect.x + (self.rect.w - txt_dims.width) / 2.;

        match self.style {
            ButtonStyle::Default => {
                let background = if hovered && !is_dropdown_open() && !self.blocked && !is_mouse_button_down(MouseButton::Left) {
                    self.hover_color
                } else {
                    FIELD_BACKGROUND_COLOR
                };
                draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, background);
                draw_rectangle_lines(self.rect.x, self.rect.y, self.rect.w, self.rect.h, 2., OUTLINE_COLOR);
            }
            ButtonStyle::Plain => {
                if hovered && !is_dropdown_open() && !self.blocked && !is_mouse_button_down(MouseButton::Left) {
                    draw_rectangle(
                        self.rect.x,
                        self.rect.y,
                        self.rect.w,
                        self.rect.h,
                        self.hover_color,
                    );
                }
            }
        }

        draw_text_ui(self.label, txt_x + self.text_offset.x, txt_y + self.text_offset.y, FIELD_TEXT_SIZE_16, self.text_color);

        is_mouse_button_pressed(MouseButton::Left)
        && hovered
        && !self.blocked
        && !is_dropdown_open()
    }
}
