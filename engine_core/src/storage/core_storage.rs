// engine_core/src/storage/core_storage.rs
use uuid::Uuid;
use crate::{
    constants::GAME_SAVE_ROOT, game::game::Game
};
use std::{
    collections::HashMap, env, fs, io, path::{Path, PathBuf}, time::SystemTime
};

pub type WorldIndex = HashMap<Uuid, String>;

/// Find all game folders in `games/`.
pub fn list_game_folders() -> io::Result<Vec<PathBuf>> {
    let root = resource_path(GAME_SAVE_ROOT);
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

/// Load the game from disk.
pub fn load_game_from_folder(folder: &Path) -> io::Result<Game> {
    let path = folder.join("game.ron");
    let ron_string = fs::read_to_string(path)?;
    ron::from_str(&ron_string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Resolve a path that is relative to the Resources directory when the
/// binary runs from a bundle, or the workspace root layout.
pub fn resource_path(rel: impl AsRef<Path>) -> PathBuf {
    // Bundle path
    if let Ok(res_dir) = env::var("CARGO_BUNDLE_RESOURCES") {
        return Path::new(&res_dir).join(rel);
    }

    // Dev mode path
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("cannot find workspace root")
        .join(rel)
}