use engine_core::ui::widgets::FIELD_TEXT_SIZE;
// editor/src/gui/menu_bar.rs
use engine_core::ui::widgets::gui_dropdown_blend;
use engine_core::ui::widgets::WidgetId;
use engine_core::ui::widgets::rect_width_for_text;
use macroquad::prelude::*;
use strum_macros::Display;
use crate::gui::gui_constants::*;
use strum_macros::EnumIter;

/// Holds the state of the top‑level menu bar.
pub struct MenuBar {
    file_id: WidgetId,
    edit_id: WidgetId,
    pub pending: Option<MenuAction>,
}

#[derive(EnumIter, Clone, Copy, PartialEq, Eq, Debug, Display)]
pub enum MenuAction {
    // File actions
    NewGame,
    Open,
    Save,
    // Edit actions
    Undo,
    Redo,
}

impl MenuAction {
    /// Returns the text that should be shown in dropdowns, lists, etc.
    pub fn ui_label(&self) -> String {
        match self {
            MenuAction::NewGame => "New Game".to_string(),
            MenuAction::Save => "Save: ctrl + S".to_string(),
            MenuAction::Undo => "Undo: ctrl + Z".to_string(),
            MenuAction::Redo => "Redo: shift + ctrl + Z".to_string(),
            _ => format!("{self:?}"),
        }
    }
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            file_id: WidgetId::default(),
            edit_id: WidgetId::default(),
            pending: None,
        }
    }

    /// Draw the menu options and return any requested action.
    pub fn draw(&mut self, title: &str) -> Option<MenuAction> {
        // Height of each dropdown item
        const HEIGHT: f32 = 30.0;
        // Offset for the dropdown options
        const DROPDOWN_Y_OFFSET: f32 = 7.5;

        // The panel is already drawn in each sub editor
        let panel_rect = menu_panel_rect(); 

        let mut x = panel_rect.x + PADDING;
        let y = panel_rect.y + PADDING / 2.0;

        let title_dims = draw_text(
            title,
            panel_rect.x + PADDING,
            panel_rect.y + 31.0,
            FIELD_TEXT_SIZE * 1.25,
            BLACK,
        );

        x += title_dims.width + SPACING * 3.0;

        // File dropdown
        let file_label = "File";

        let file_rect = Rect::new(
            x, 
            y, 
            rect_width_for_text(file_label), 
            HEIGHT
        );

        let file_actions: Vec<MenuAction> = vec![
            MenuAction::NewGame,
            MenuAction::Open,
            MenuAction::Save,
        ];

        if let Some(selected) = gui_dropdown_blend(
            self.file_id,
            file_rect,
            file_label,
            &file_actions,
            |a| a.ui_label(),
            BLACK,
            DROPDOWN_Y_OFFSET
        ) {
            self.pending = Some(selected);
        }

        x += file_rect.w + SPACING;

        // Edit dropdown
        let edit_label = "Edit";

        let edit_rect = Rect::new(
            x, 
            y, 
            rect_width_for_text(edit_label), 
            HEIGHT
        );

        let edit_actions: Vec<MenuAction> = vec![
            MenuAction::Undo,
            MenuAction::Redo,
        ];

        if let Some(selected) = gui_dropdown_blend(
            self.edit_id,
            edit_rect,
            edit_label,
            &edit_actions,
            |a| a.ui_label(),
            BLACK,
            DROPDOWN_Y_OFFSET
        ) {
            self.pending = Some(selected);
        }

        // Return the action
        self.pending.take()
    }
}

/// Draws a the panel background for the top menu across the whole width of the screen and returns its `Rect`.
pub fn draw_top_panel_full() -> Rect {
    let rect = menu_panel_rect();
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_COLOR);
    rect
}

fn menu_panel_rect() -> Rect {
    Rect::new(0.0, 0.0, screen_width(), MENU_PANEL_HEIGHT)
}