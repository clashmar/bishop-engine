// engine_core/src/ui/prompt.rs
use crate::controls::controls::Controls;
use crate::ui::widgets::*;
use crate::ui::text::*;
use macroquad::prelude::*;
use crate::ui::widgets::WidgetId;

pub enum StringPromptResult {
    Confirmed(String),
    Cancelled,
}

/// A prompt that draws:
///   * Message line,
///   * Text field,
///   * Confirm / Cancel buttons.
pub struct StringPromptWidget {
    /// Unique id for the text field.
    input_id: WidgetId,
    /// Rectangle that contains the whole widget.
    rect: Rect,
    /// Message shown above the text field.
    message: String,
    /// Current contents of the text field.
    current: String,
}

impl StringPromptWidget {
    /// Create a new prompt centred inside the supplied rect.
    pub fn new(modal_rect: Rect, message: impl Into<String>) -> Self {
        const FIELD_H: f32 = 30.0;
        const BUTTON_H: f32 = 30.0;
        const GAP: f32 = 10.0;

        let inner_w = modal_rect.w * 0.8;
        let inner_x = modal_rect.x + (modal_rect.w - inner_w) / 2.0;

        let total_h = 20.0 + FIELD_H + GAP + BUTTON_H;
        let inner_y = modal_rect.y + (modal_rect.h - total_h) / 2.0;

        let rect = Rect::new(inner_x, inner_y, inner_w, total_h);

        Self {
            input_id: WidgetId::default(),
            rect,
            message: message.into(),
            current: String::new(),
        }
    }

    /// Draws the widget and, return the result if confirmed/cancelled or None.
    pub fn draw(&mut self) -> Option<StringPromptResult> {
        // Message
        let message_pos = vec2(self.rect.x, self.rect.y + 10.0);

        draw_text_ui(
            &self.message,
            message_pos.x,
            message_pos.y,
            DEFAULT_FONT_SIZE_16,
            WHITE,
        );

        // Text field
        let field_rect = Rect::new(
            self.rect.x,
            self.rect.y + 20.0,
            self.rect.w,
            30.0,
        );

        let (new_text, _) = gui_input_text_focused(self.input_id, field_rect, &self.current);
        self.current = new_text;

        // Buttons
        let btn_y = self.rect.y + 70.0;
        let btn_w = 80.0;
        let btn_h = 30.0;
        let spacing = (self.rect.w - 2.0 * btn_w) / 3.0;

        let confirm_rect = Rect::new(
            self.rect.x + spacing,
            btn_y,
            btn_w,
            btn_h,
        );

        let cancel_rect = Rect::new(
            self.rect.x + 2.0 * spacing + btn_w,
            btn_y,
            btn_w,
            btn_h,
        );

        let confirm_clicked = gui_button(confirm_rect, "Confirm");
        let cancel_clicked = gui_button(cancel_rect, "Cancel");

        // Handle result
        if (confirm_clicked || Controls::enter())
        && !self.current.trim().is_empty()  {
            return Some(StringPromptResult::Confirmed(self.current.clone()));
        }

        if cancel_clicked || Controls::escape() {
            return Some(StringPromptResult::Cancelled);
        }

        None
    }
}