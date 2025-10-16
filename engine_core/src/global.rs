// engine_core/src/global.rs
use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;
use crate::constants::DEFAULT_TILE_SIZE;

static TILE_SIZE: OnceCell<Arc<Mutex<f32>>> = OnceCell::new();

/// Sets the global tile size.  
pub fn set_tile_size(size: f32) {
    let size = size.max(0.0).max(DEFAULT_TILE_SIZE);
    if let Some(cell) = TILE_SIZE.get() {
        *cell.lock().unwrap() = size;
    } else {
        let _ = TILE_SIZE.set(Arc::new(Mutex::new(size)));
    }
}

/// Returns the tile size of the active game, or the default if not initialized.
pub fn tile_size() -> f32 {
    match TILE_SIZE.get() {
        Some(cell) => *cell.lock().unwrap(),
        None => DEFAULT_TILE_SIZE,
    }
}