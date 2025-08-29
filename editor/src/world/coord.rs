use macroquad::prelude::*;
use core::constants::*;

/// Convert the current mouse position (screen pixels) to world
/// coordinates using the supplied camera.
pub fn mouse_world_pos(camera: &Camera2D) -> Vec2 {
    let (x, y) = mouse_position();               
    camera.screen_to_world(vec2(x, y))  
}

/// Snap an world‑space point to the integer grid that the
/// editor works with.
pub fn snap_to_grid(pos: Vec2) -> Vec2 {
    vec2(pos.x.floor(), pos.y.floor())
}

/// Return the grid cell (integer coordinates) that the mouse is
/// hovering over.
pub fn mouse_world_grid(camera: &Camera2D) -> Vec2 {
    let world = mouse_world_pos(camera);
    (world / TILE_SIZE).floor()
}