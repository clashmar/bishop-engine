use engine_core::{assets::asset_manager::{AssetManager}, tiles::{tile::Tile, tilemap::TileMap}, world::{room::RoomMetadata, world::World}};
use std::{future::Future, pin::Pin};
use macroquad::prelude::*;

pub trait WorldUiElement {
    fn draw(&self, world: &World);
    fn on_click<'a>(&'a self, world: &'a mut World) -> Pin<Box<dyn Future<Output=()> + Send + 'a>>;
    fn rect(&self, _world: &World) -> Option<Rect> { None } // default None
}

pub trait DynamicTilemapUiElement {
    fn draw(&self, camera: &Camera2D);
    fn is_mouse_over(&self, mouse_pos: Vec2, camera: &Camera2D) -> bool;
    fn on_click(
        &mut self, 
        map: &mut TileMap, 
        room_metadata: &mut RoomMetadata,
        mouse_pos: Vec2, 
        camera: &Camera2D,
        other_bounds: &[(Vec2, Vec2)]
    );
}

pub trait TilemapUiElement {
    fn draw(
        &mut self, 
        camera: &Camera2D,
        asset_manager: &mut AssetManager,
    );
    fn is_mouse_over(&self, mouse_pos: Vec2, camera: &Camera2D) -> bool;
    fn on_click(
        &mut self,
        selected_tile: &mut Tile, 
        mouse_pos: Vec2, 
        camera: &Camera2D,
    );
}


