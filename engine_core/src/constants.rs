// engine_core/src/constants.rs
use macroquad::prelude::*;

/// Base tile size for editor scaling.
pub const BASE_TILE_SIZE: f32 = 32.0;

/// Actual tile size that the world scales to.
pub const TILE_SIZE: f32 = 16.0;

pub const DEFAULT_ROOM_SIZE: Vec2 = vec2(10.0, 5.0);
pub const DEFAULT_ROOM_POSITION: Vec2 = vec2(0.0, 0.0);

pub const GAME_SAVE_ROOT: &str = "games";

/// Scale to the base resolution.
pub const EDITOR_ZOOM_FACTOR: f32 = TILE_SIZE / BASE_TILE_SIZE;

pub const CAMERA_TILES_X: f32 = 55.0;     
pub const CAMERA_TILES_Y: f32 = 35.0;

pub const WORLD_VIRTUAL_WIDTH:  f32 = CAMERA_TILES_X * TILE_SIZE;
pub const WORLD_VIRTUAL_HEIGHT: f32 = CAMERA_TILES_Y * TILE_SIZE;

pub const FIXED_WINDOW_WIDTH:  i32 = (CAMERA_TILES_X * BASE_TILE_SIZE) as i32;
pub const FIXED_WINDOW_HEIGHT: i32 = (CAMERA_TILES_Y * BASE_TILE_SIZE) as i32;

// Prevents the window from becoming absurdly small/large
pub const MIN_WINDOW_WIDTH:  i32 = 640;
pub const MIN_WINDOW_HEIGHT: i32 = 360;
pub const MAX_WINDOW_WIDTH:  i32 = 1366;
pub const MAX_WINDOW_HEIGHT: i32 = 768;