// editor/src/storage/path_utils.rs
use std::path::PathBuf;
use engine_core::constants::*;

// TODO: Make this OS agnostic future proof
/// Returns the absolute path to the folder that stores all games.
pub fn absolute_save_root() -> PathBuf {
    let root_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    let mut dir = root_dir;
    dir.push(GAME_SAVE_ROOT);
    dir
}

/// Turns a game name into a safe folder name.
pub fn sanitise_name(name: &str) -> String {
    let trimmed = name.trim_matches(|c: char| c.is_whitespace());
    let mut out = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '_'         
            }
        })
        .collect::<String>();

    // Collapse consecutive underscores
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

/// Path to the folder that belongs to a particular game.
pub fn game_folder(name: &str) -> PathBuf {
    absolute_save_root().join(sanitise_name(name))
}

/// Path to the assets folder inside a game folder.
pub fn assets_folder(name: &str) -> PathBuf {
    game_folder(name).join("assets")
}