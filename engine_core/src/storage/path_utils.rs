// engine_core/src/storage/path_utils.rs
use std::path::Path;
use std::path::PathBuf;
use crate::constants::GAME_SAVE_ROOT;

/// Returns the absolute path to the folder that stores all games.
pub fn absolute_save_root() -> PathBuf {
    // Bundled binary
    if let Ok(res_dir) = std::env::var("CARGO_BUNDLE_RESOURCES") {
        return PathBuf::from(res_dir).join(GAME_SAVE_ROOT);
    }

    // Dev mode
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent() // editor
        .expect("Cannot locate workspace root.");

    workspace_root.join(GAME_SAVE_ROOT)
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

/// Returns `Ok(())` if `candidate` is inside `absolute_save_root()`.
pub fn ensure_inside_save_root(path: &Path) -> Result<(), String> {
    let root = absolute_save_root()
        .canonicalize()
        .map_err(|e| format!("Cannot canonicalize save root: {e}"))?;
    let candidate = path
        .canonicalize()
        .map_err(|e| format!("Cannot canonicalize selected folder: {e}"))?;

    if candidate.starts_with(&root) {
        Ok(())
    } else {
        Err(format!(
            "Selected folder '{}' is not in the 'games' directory '{}'.",
            candidate.display(),
            root.display()
        ))
    }
}
