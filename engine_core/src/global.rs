// engine_core/src/global.rs
use crate::{constants::{DEFAULT_TILE_SIZE, MINIMUM_TILE_SIZE}, ecs::component::Position, game::game::Game};
use std::sync::Mutex;

static TILE_SIZE: Mutex<f32> = Mutex::new(DEFAULT_TILE_SIZE);

/// Sets the global tile size. Call when creating/loading a game.
pub fn set_global_tile_size(size: f32) {
    let mut guard = TILE_SIZE.lock().unwrap();
    *guard = size.max(0.0).max(MINIMUM_TILE_SIZE);
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

        let pos_store = world.world_ecs.get_store_mut::<Position>();
        for (_entity, pos) in &mut pos_store.data {
            pos.position *= sf;
        }
    }
}

/// Returns the tile size of the active game, or the default if not initialized.
pub fn tile_size() -> f32 {
    *TILE_SIZE.lock().unwrap()
}