// engine_core/src/global.rs
use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::{constants::DEFAULT_TILE_SIZE, game::game::Game};

/// Holds an immutable reference to the currently active `Game`.
static CURRENT_GAME: OnceCell<Arc<Game>> = OnceCell::new();

/// Initialise the global game that the editor has just loaded/created.
pub fn set_current_game(game: Game) {
    let _ = CURRENT_GAME.set(Arc::new(game));
}

/// Returns a shared reference to the active game.
pub fn current_game() -> Arc<Game> {
    CURRENT_GAME
        .get()
        .expect("CURRENT_GAME has not been initialised.")
        .clone()
}

/// Returns the tile size of the active game, or the default if not initialized.
pub fn tile_size() -> f32 {
    let game = match CURRENT_GAME.get() {
        Some(g) => g.clone(),
        None => return DEFAULT_TILE_SIZE,
    };

    game.tile_size
        .max(0.0)
        .max(DEFAULT_TILE_SIZE)
}