// editor/src/canvas/grid.rs
use crate::{
    canvas::grid_shader::{draw_shader_grid, GridParams},
    editor_camera_controller::{self, EditorCameraController},
    world::world_editor::LINE_THICKNESS_MULTIPLIER,
};
use macroquad::prelude::*;

const GRID_LINE_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.2);

/// Draw a grid overlay for the editor using a shader.
pub fn draw_grid(camera: &Camera2D, grid_size: f32) {
    let scalar = EditorCameraController::scalar_zoom(camera);
    if scalar < editor_camera_controller::MIN_ZOOM * 4.0 {
        return;
    }

    let params = GridParams {
        camera_pos: camera.target,
        camera_zoom: scalar,
        viewport_size: vec2(screen_width(), screen_height()),
        grid_size,
        line_color: GRID_LINE_COLOR,
        line_thickness: LINE_THICKNESS_MULTIPLIER / 2.0,
    };

    draw_shader_grid(&params);
}
