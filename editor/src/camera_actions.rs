use macroquad::prelude::*;
use core::constants::*;

pub fn camera_for_room(room_size: Vec2, room_position: Vec2) -> Camera2D {
    let max_dim_px = (room_size * TILE_SIZE).max_element(); 
    let zoom = WORLD_EDITOR_ZOOM_FACTOR / max_dim_px;       

    Camera2D {
        target: (room_position + room_size / 2.0) * TILE_SIZE,
        zoom: vec2(zoom, zoom),
        ..Default::default()
    }
}