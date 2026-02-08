// editor/src/canvas/grid.rs
use crate::{
    editor_camera_controller::{self, EditorCameraController},
    world::world_editor::LINE_THICKNESS_MULTIPLIER,
};
use macroquad::prelude::*;

const GRID_LINE_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.2);

/// Draw a grid overlay for the editor.
pub fn draw_grid(camera: &Camera2D, grid_size: f32) {
    let scalar = EditorCameraController::scalar_zoom(camera);
    if scalar < editor_camera_controller::MIN_ZOOM * 4.0 {
        return;
    }

    let line_thickness = (LINE_THICKNESS_MULTIPLIER / 2.0) / scalar;

    let cam_pos = camera.target;
    let screen_w = screen_width() / scalar;
    let screen_h = screen_height() / scalar;

    // start_x / start_y are the first grid lines that are left / top of the view.
    let start_x = ((cam_pos.x - screen_w / 2.0) / grid_size).floor() * grid_size;
    let start_y = ((cam_pos.y - screen_h / 2.0) / grid_size).floor() * grid_size;
    // end_x / end_y extend a little beyond the view so the last line is drawn.
    let end_x = cam_pos.x + screen_w / 2.0 + grid_size;
    let end_y = cam_pos.y + screen_h / 2.0 + grid_size;

    // Draw vertical lines.
    let mut x = start_x;
    while x <= end_x {
        draw_line(x, start_y, x, end_y, line_thickness, GRID_LINE_COLOR);
        x += grid_size;
    }

    // Draw horizontal lines.
    let mut y = start_y;
    while y <= end_y {
        draw_line(start_x, y, end_x, y, line_thickness, GRID_LINE_COLOR);
        y += grid_size;
    }
}