use macroquad::prelude::*;

pub const TILE_SIZE: f32 = 32.0;
pub const PLAYER_WIDTH: f32 = 60.0;
pub const PLAYER_HEIGHT: f32 = 80.0;
pub const GRAVITY: f32 = 0.4;

pub const DEFAULT_ROOM_SIZE: Vec2 = vec2(10.0, 5.0);
pub const DEFAULT_ROOM_POSITION: Vec2 = vec2(0.0, 0.0);

pub const WORLD_SAVE_FOLDER: &str = "assets/worlds";
pub const PREFAB_SAVE_FOLDER: &str = "assets/prefabs";

pub const WORLD_EDITOR_ZOOM_FACTOR: f32 = 1.0;
pub const WORLD_VIRTUAL_WIDTH: f32 = 800.0;
pub const WORLD_VIRTUAL_HEIGHT: f32 = 600.0;

