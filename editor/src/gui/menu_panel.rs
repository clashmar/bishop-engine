// editor/src/gui/menu_panel.rs
use macroquad::prelude::*;
use crate::gui::gui_constants::*;

/// Draws a the panel background for the top menu across the whole width of the screen and returns its `Rect`.
pub fn draw_top_panel_full() -> Rect {
    let rect = Rect::new(0.0, 0.0, screen_width(), MENU_PANEL_HEIGHT);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_COLOR);
    rect
}