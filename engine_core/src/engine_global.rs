// engine_core/src/engine_global.rs
use crate::ecs::position::Position;
use crate::game::game::Game;
use crate::constants::*;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineMode {
    Editor,
    Game
}

pub static ENGINE_MODE: Mutex<EngineMode> = Mutex::new(EngineMode::Editor);

/// Switch the global engine mode.
pub fn set_engine_mode(mode: EngineMode) {
    *ENGINE_MODE.lock().unwrap() = mode;
}

/// Retrieve the current engine mode.
pub fn get_engine_mode() -> EngineMode {
    *ENGINE_MODE.lock().unwrap()
}

static TILE_SIZE: Mutex<f32> = Mutex::new(DEFAULT_TILE_SIZE);
static CAM_TILE_DIMS: Mutex<(f32, f32)> = Mutex::new((DEFAULT_CAM_TILES_X, DEFAULT_CAM_TILES_Y));

/// Returns the tile size of the active game, or the default if not initialized.
pub fn tile_size() -> f32 {
    *TILE_SIZE.lock().unwrap()
}

/// Returns the width and height of the game virtual screen in terms of grid tiles, 
/// or the default if not initialized.
pub fn cam_tile_dims() -> (f32, f32) {
    *CAM_TILE_DIMS.lock().unwrap()
}

/// Sets the global tile size. Call when creating/loading a game.
pub fn set_global_tile_size(size: f32) {
    let mut guard = TILE_SIZE.lock().unwrap();
    *guard = size.max(0.0).max(MINIMUM_TILE_SIZE);
}

/// Sets the global tile size. Call when creating/loading a game.
pub fn set_global_cam_tile_dims(dims: (f32, f32)) {
    let mut guard = CAM_TILE_DIMS.lock().unwrap();
    let x = dims.0.max(1.0);
    let y = dims.1.max(1.0);
    *guard = (x, y);
}

/// Updates the global tile size and entity positions.
pub fn update_tile_size(game: &mut Game, old_size: f32, new_size: f32) {
    let old = if old_size <= 0.0 { DEFAULT_TILE_SIZE } else { old_size };
    let new = new_size.max(1.0).max(MINIMUM_TILE_SIZE);

    game.tile_size = new;
    set_global_tile_size(new);  

    let sf = new / old; 

    for world in &mut game.worlds {
        for room in &mut world.rooms {
            room.position *= sf;
        }
    }

    let pos_store = game.ecs.get_store_mut::<Position>();
    for (_entity, pos) in &mut pos_store.data {
        pos.position *= sf;
    }
}
