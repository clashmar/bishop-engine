// engine_core/src/ui/prompt.rs
use crate::assets::asset_manager::AssetManager;
use crate::assets::sprite::SpriteId;
use crate::controls::controls::Controls;
use crate::ui::widgets::*;
use crate::ui::text::*;
use crate::world::world::WorldId;
use macroquad::prelude::*;
use crate::ui::widgets::WidgetId;

pub const BUTTON_W: f32 = 80.0;
pub const BUTTON_H: f32 = 30.0;
pub const BUTTON_SPACING: f32 = 10.0;
pub const FIELD_H: f32 = 30.0;

/// Result of a string prompt.
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
        let (confirm_rect, cancel_rect) = confirm_cancel_rects(self.rect, btn_y);
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

/// Result of a confirm prompt.
pub enum ConfirmPromptResult {
    Confirmed,
    Cancelled,
}

/// A prompt that draws:
///   * Message line,
///   * Confirm / Cancel buttons.
pub struct ConfirmPromptWidget {
    /// Rectangle that contains the whole widget.
    rect: Rect,
    /// Message to display.
    message: String,
}

impl ConfirmPromptWidget {
    /// Create a new prompt centred inside the supplied rect.
    pub fn new(modal_rect: Rect, message: impl Into<String>) -> Self {
        let inner_w = modal_rect.w * 0.8;
        let inner_x = modal_rect.x + (modal_rect.w - inner_w) / 2.0;

        let total_h = modal_rect.h * 1.3;
        let inner_y = modal_rect.y + (total_h - modal_rect.h);

        let rect = Rect::new(inner_x, inner_y, inner_w, total_h);

        Self {
            rect,
            message: message.into(),
        }
    }

    /// Draws the widget and, return the result if confirmed/cancelled or None.
    pub fn draw(&mut self) -> Option<ConfirmPromptResult> {
        // Message
        let center_x = self.rect.x + (self.rect.w / 2.0);
        let message_x = center_text(center_x, &self.message, DEFAULT_FONT_SIZE_16).0;
        let message_height = measure_text_ui(&self.message, DEFAULT_FONT_SIZE_16, 1.0).height;

        draw_text_ui(
            &self.message,
            message_x,
            self.rect.y,
            DEFAULT_FONT_SIZE_16,
            WHITE,
        );

        // Buttons
        let btn_y = self.rect.y + message_height + WIDGET_SPACING;
        let (confirm_rect, cancel_rect) = confirm_cancel_rects(self.rect, btn_y);
        let confirm_clicked = gui_button(confirm_rect, "Confirm");
        let cancel_clicked = gui_button(cancel_rect, "Cancel");

        // Handle result
        if confirm_clicked || Controls::enter() {
            return Some(ConfirmPromptResult::Confirmed);
        }

        if cancel_clicked || Controls::escape() {
            return Some(ConfirmPromptResult::Cancelled);
        }

        None
    }
}

/// Result an edit world prompt.
pub struct WorldEditResult {
    pub id: WorldId,
    pub name: Option<String>,
    pub sprite: Option<SpriteId>,
}

/// Prompt that draws:
///   * Edit name text field,
///   * World sprite picker,
///   * Confirm / Cancel buttons.
pub struct WorldEditPrompt {
    world_id: WorldId,
    name_id: WidgetId,
    rect: Rect,
    og_name: String,
    og_sprite: SpriteId,
    current_name: String,
    current_sprite: SpriteId,
}

impl WorldEditPrompt {
    /// Create a new prompt centred inside the supplied rect.
    pub fn new(
        world_id: WorldId,
        modal_rect: Rect,
        name_id: WidgetId,
        og_name: impl Into<String>,
        og_sprite: SpriteId,
    ) -> Self {
        let inner_w = modal_rect.w * 0.8;
        let inner_x = modal_rect.x + (modal_rect.w - inner_w) / 2.0;

        let total_h = modal_rect.h * 1.225;
        let inner_y = modal_rect.y + (total_h - modal_rect.h);

        let rect = Rect::new(inner_x, inner_y, inner_w, total_h);

        let name = og_name.into();

        Self {
            world_id,
            name_id: name_id,
            rect,
            og_name: name.clone(),
            og_sprite,
            current_name: name,
            current_sprite: og_sprite,
        }
    }

    /// Draws the widget and, return the result if confirmed/cancelled or None.
    pub fn draw(&mut self, asset_manager: &mut AssetManager) -> Option<WorldEditResult> {
        const GAP: f32 = 5.0;
        const FIELD_GAP:f32 = 30.0;

        let mut y = self.rect.y;

        // Name label
        let mut label_dims = draw_text_ui(
            "Edit name:",
            self.rect.x,
            y,
            DEFAULT_FONT_SIZE_16,
            WHITE,
        );

        y += label_dims.height + GAP;

        // Name field
        let name_rect = Rect::new(self.rect.x, y, self.rect.w, FIELD_H);
        let (new_name, _) = gui_input_text_clamped(self.name_id, name_rect, &self.current_name, 33);
        self.current_name = new_name;

        y += name_rect.h + FIELD_GAP;

        // Sprite label
        label_dims = draw_text_ui(
            "Change sprite:",
            self.rect.x,
            y,
            DEFAULT_FONT_SIZE_16,
            WHITE,
        );

        y += label_dims.height + GAP;

        let sprite_rect = Rect::new(self.rect.x, y, self.rect.w, 30.0);
        if gui_sprite_picker(sprite_rect, &mut self.current_sprite, asset_manager) {
            // Widget updates the sprite
        }

        y += sprite_rect.h + FIELD_GAP;

        // Buttons
        let (confirm_rect, cancel_rect) = confirm_cancel_rects(self.rect, y);
        let confirm_clicked = gui_button(confirm_rect, "Confirm");
        let cancel_clicked = gui_button(cancel_rect, "Cancel");

        // Result
        if (confirm_clicked || Controls::enter())
            && !self.current_name.trim().is_empty()
        {
            // Build a result that only contains the fields the user actually changed
            let name = if self.current_name != self.og_name {
                Some(self.current_name.clone())
            } else {
                None
            };
            let sprite = if self.current_sprite != self.og_sprite {
                Some(self.current_sprite)
            } else {
                None
            };
            return Some(WorldEditResult { id: self.world_id, name, sprite });
        }

        if cancel_clicked || Controls::escape() {
            return Some(WorldEditResult { id: self.world_id, name: None, sprite: None });
        }

        None
    }
}

/// Supplies centered rects for confirm/cancel buttons.
fn confirm_cancel_rects(rect: Rect, btn_y: f32) -> (Rect, Rect) {
    let spacing = (rect.w - 2.0 * BUTTON_W) / 3.0;
    let confirm_rect = Rect::new(rect.x + spacing, btn_y, BUTTON_W, BUTTON_H);
    let cancel_rect = Rect::new(rect.x + 2.0 * spacing + BUTTON_W, btn_y, BUTTON_W, BUTTON_H);
    (confirm_rect, cancel_rect)
}