// engine_core/src/storage/core_storage.rs
use crate::storage::path_utils::*;
use crate::game::game::Game;
use std::io::ErrorKind;
use std::io::Error;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::io;
use std::time::SystemTime;
use uuid::Uuid;
use std::collections::HashMap;

pub type WorldIndex = HashMap<Uuid, String>;

/// Find all game folders in `games/`.
pub fn list_game_folders() -> io::Result<Vec<PathBuf>> {
    let root = absolute_save_root();
    let mut folders = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("game.ron").exists() {
            folders.push(path);
        }
    }

    Ok(folders)
}

/// Return the most recently modified game folder.
pub fn most_recent_game_folder() -> Option<PathBuf> {
    let mut best: Option<(PathBuf, SystemTime)> = None;

    for path in list_game_folders().ok()? {
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mod_time) = meta.modified() {
                match best {
                    None => best = Some((path.clone(), mod_time)),
                    Some((_, t)) if mod_time > t => best = Some((path.clone(), mod_time)),
                    _ => {}
                }
            }
        }
    }

    best.map(|(p, _)| p)
}

/// Finds the game .ron in /Resources and returns an initialized `Game`.
pub async fn load_game_ron() -> io::Result<Game> {
    match resources_dir_from_exe() {
        Some(resources_folder) => {
            match load_game_from_folder(&resources_folder).await {
                Ok(game) => Ok(game),
                Err(err) => Err(err),
            }
        },
        None => Err(Error::new(ErrorKind::Other, "Could not find resources."))
    }
}

/// Load the game .ron from a specified folder.
pub async fn load_game_from_folder(folder: &Path) -> io::Result<Game> {
    let path = folder.join("game.ron");
    let ron_string = fs::read_to_string(path)?;

    // Parse the RON
    match ron::from_str::<Game>(&ron_string) {
        Ok(mut game) => {
            game.initialize().await;
            Ok(game)
        },
        // Corrupt file
        Err(e) => Err(Error::new(ErrorKind::Other, e))
    }
}