// editor/src/gui/menu_panel.rs
use macroquad::prelude::*;
use crate::gui::gui_constants::*;



pub fn draw_panel_background() {
    draw_rectangle(0.0, 0.0, screen_width(), MENU_PANEL_HEIGHT, PANEL_COLOR);
}