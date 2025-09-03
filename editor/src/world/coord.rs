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

/// Turn a world‑space `Vec2` into screen coordinates using the current camera.
pub fn world_to_screen(camera: &Camera2D, world_pos: Vec2) -> Vec2 {
    camera.world_to_screen(world_pos)
}

pub fn overlaps_existing_rooms(
    pos: Vec2,
    size: Vec2,
    other_bounds: &[(Vec2, Vec2)],
) -> bool {
    let a_min = pos;
    let a_max = pos + size;

    other_bounds.iter().any(|(b_pos, mut b_size)| {
        b_size *= TILE_SIZE;

        let b_min = *b_pos;
        let b_max = *b_pos + b_size;

        a_min.x < b_max.x &&
        a_max.x > b_min.x &&
        a_min.y < b_max.y &&
        a_max.y > b_min.y
    })
}