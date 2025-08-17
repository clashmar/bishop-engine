use core::{tile::Tile, tilemap::TileMap};
use macroquad::prelude::*;

pub trait UiElement {
    fn draw(&self, camera: &Camera2D);
    fn is_mouse_over(&self, mouse_pos: Vec2, camera: &Camera2D) -> bool;
    fn on_click(
        &mut self, 
        map: &mut TileMap, 
        room_size: &mut Vec2, 
        room_position: &mut Vec2, 
        selected_tile: &mut Tile, 
        mouse_pos: Vec2, 
        camera: &Camera2D,
        other_bounds: &[(Vec2, Vec2)]
    );
}


