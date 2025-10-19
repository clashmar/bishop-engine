// engine_core/src/constants.rs
use macroquad::prelude::*;
use crate::global::tile_size;

/// 60Hz pysics.
pub const FIXED_DT: f32 = 1.0 / 60.0;
/// Protects against long freezes.
pub const MAX_ACCUM: f32 = 0.25; 

/// Default tile size that the world scales to.
pub const DEFAULT_TILE_SIZE: f32 = 16.0;
pub const MINIMUM_TILE_SIZE: f32 = 9.0;

/// Base tile size for editor scaling.
pub const BASE_TILE_SIZE: f32 = 32.0;

pub const DEFAULT_ROOM_SIZE: Vec2 = vec2(16.0, 10.0);
pub const DEFAULT_ROOM_POSITION: Vec2 = vec2(0.0, 0.0);

pub const GAME_SAVE_ROOT: &str = "games";

/// Scale to the base resolution.
pub fn editor_zoom_factor() -> f32 { tile_size() / BASE_TILE_SIZE }

pub const DEFAULT_CAM_TILES_X: f32 = 16.0;     
pub const DEFAULT_CAM_TILES_Y: f32 = 10.0;

pub const FIXED_WINDOW_WIDTH:  i32 = (DEFAULT_CAM_TILES_X * 3. * BASE_TILE_SIZE) as i32;
pub const FIXED_WINDOW_HEIGHT: i32 = (DEFAULT_CAM_TILES_Y * 3. * BASE_TILE_SIZE) as i32;

// Prevents the window from becoming absurdly small/large
pub const MIN_WINDOW_WIDTH:  i32 = 640;
pub const MIN_WINDOW_HEIGHT: i32 = 360;
pub const MAX_WINDOW_WIDTH:  i32 = 1366;
pub const MAX_WINDOW_HEIGHT: i32 = 768;