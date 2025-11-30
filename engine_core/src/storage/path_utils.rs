// engine_core/src/storage/path_utils.rs
use std::io::ErrorKind;
use std::io::Error;
use std::io;
use futures::executor::block_on;
use macroquad::prelude::*;
use rfd::FileDialog;
use crate::constants::*;
use crate::global::EngineMode;
use crate::global::get_engine_mode;
use crate::storage::editor_config::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use crate::*; 

/// Path to the folder that belongs to a particular game (Editor).
pub fn game_folder(name: &str) -> PathBuf {
    absolute_save_root().join(sanitise_name(name))
}

/// Path to the resources folder for a game (Editor/Game).
pub fn resources_folder(name: &str) -> PathBuf {
    match get_engine_mode() {
        EngineMode::Editor => {
            game_folder(name).join(RESOURCES_FOLDER)
        }
        EngineMode::Game => {
            // Panic is acceptable here as there is no possible fallback 
            resources_dir_from_exe().unwrap()
        }
    }
}

/// Path to the assets folder inside a resources folder (Editor/Game).
pub fn assets_folder(name: &str) -> PathBuf {
    resources_folder(name).join(ASSETS_FOLDER)
}

/// Path to the windows folder inside a resources folder (Editor).
pub fn windows_folder(name: &str) -> PathBuf {
    game_folder(name).join(WINDOWS_FOLDER)
}

/// Path to the mac_os folder inside a resources folder (Editor).
pub fn mac_os_folder(name: &str) -> PathBuf {
    game_folder(name).join(MAC_OS_FOLDER)
}

/// Returns the absolute path to the folder that stores all games for the editor,
/// or the parent of the resources folder for games on all platforms.
pub fn absolute_save_root() -> PathBuf {
    // Game path
    if get_engine_mode() == EngineMode::Game {
        let path = exe_dir().unwrap_or_else(|| {
            // If this isn't found then the game can't work
            onscreen_error!("Could not find exe_dir in game mode");
            panic!("Could not find exe_dir in game mode");
        });
        return path;
    }

    // Editor dev mode
    if cfg!(debug_assertions) {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .expect("Cannot locate workspace root.");

        let path_buf = workspace_root.join(GAME_SAVE_ROOT);
        return path_buf;
    }

    // Editor release mode
    if let Some(user_path) = get_save_root() {
        // Ensure the folder still exists or recreate it
        if let Err(e) = fs::create_dir_all(&user_path) {
            onscreen_error!("Could not create user save root '{}': {e}", user_path.display());
        } else {
            return user_path;
        }

        onscreen_error!("Stored save root is no longer valid, resetting.");
        {
            // TODO: Get rid of expect
            let mut cfg = EDITOR_CONFIG.write().expect("Failed to lock CONFIG for writing.");
            cfg.save_root = None;
        }
        
        // Update the .ron
        if let Err(e) = save_config() {
            onscreen_error!("Error saving config: {e}.");
        }
    }
    else {
        // Save root needs to be set
        if let Some(path_buf) = block_on(pick_save_root_async()) {
            return path_buf;
        }
    }

    // Fallback to the platform‑default location.
    let fallback_path = default_save_root();
    let _ = fs::create_dir_all(&fallback_path);
    onscreen_error!("Using fallback save root: {}", fallback_path.display());
    fallback_path
}

// Gets the dir that contains the current process.
pub fn exe_dir() -> Option<PathBuf> {
    // Path of the running executable
    let exe = std::env::current_exe().ok()?;

    // Platform specific layout
    #[cfg(target_os = "macos")]
    {
        // …/.app/Contents/MacOS/<app>
        exe.parent().map(|p| p.to_path_buf())
    }
    // Linux is yet to be implemented
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // …/<app>.exe
        exe.parent().map(|p| p.to_path_buf())
    }
}

// Gets the Resources dir for the current process.
pub fn resources_dir_from_exe() -> Option<PathBuf> {
    // Path of the running executable dir
    let exe_dir = exe_dir()?;

    // Platform specific layout
    #[cfg(target_os = "macos")]
    {
        // …/Bishop.app/Contents/MacOS/
        return exe_dir.parent() // Contents/
            .map(|p| p.join(RESOURCES_FOLDER)); // Resources/
    }
    // Linux is yet to be implemented
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // …/Bishop.exe  or  …/bishop
        Some(exe_dir.join(RESOURCES_FOLDER))
    }
}

/// Pick the folder that will become the absolute save root.
pub async fn pick_save_root_async() -> Option<PathBuf> {
    // Let the user choose a base folder
    let base_folder = FileDialog::new()
        .set_title("Select a folder for the editor assets root directory.")
        .pick_folder()
        .unwrap_or_else(|| default_save_root());

    // Build the full path
    let save_root = base_folder
        .join("Bishop Engine")
        .join(GAME_SAVE_ROOT);

    // Make sure the directory chain exists
    if let Err(e) = fs::create_dir_all(&save_root) {
        onscreen_error!("Cannot write to the selected folder: {e}");
        return None;
    }

    // Update the in memory config
    {
        let mut cfg = EDITOR_CONFIG.write().expect("Failed to lock CONFIG for writing");
        cfg.save_root = Some(save_root.clone());
    }
    
    // Update the .ron
    if let Err(e) = save_config() {
        onscreen_error!("Error saving config: {e}.");
    }

    onscreen_info!("Successfully created save root at: {:?}", save_root);
    Some(save_root)
}


/// Checks for a valid save root, or prompts the user to choose one.
pub async fn ensure_save_root() -> bool {
    // Fast path
    if get_save_root().is_some() {
        return true;
    }

    // Give Macroquad a chance to start its event loop
    next_frame().await;

    // Show the async picker.
    if let Some(_path) = pick_save_root_async().await {
        return get_save_root().is_some();
    }

    // The user cancelled the picker
    false
}

/// Turns a game name into a safe folder name.
pub fn sanitise_name(name: &str) -> String {
    let trimmed = name.trim_matches(|c: char| c.is_whitespace());
    let mut out = trimmed
        .chars()
        .map(|c| {
            // keep spaces and TODO: decide other special chars
            if c.is_ascii_alphanumeric() || c == ' ' {
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

/// Returns `Ok(())` if `candidate` is inside `absolute_save_root()`.
pub fn ensure_inside_save_root(path: &Path) -> Result<(), String> {
    let root = absolute_save_root();

    if path.starts_with(&root) {
        Ok(())
    } else {
        Err(format!(
            "Selected folder '{}' is not in the 'games' directory '{}'.",
            path.display(),
            root.display()
        ))
    }
}

/// Recursively copy the directory.
pub fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
    if !src.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Source `{}` is not a directory.", src.display()),
        ));
    }

    // Create the target directory
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Platform-default location used when the user has not chosen a folder.
fn default_save_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").expect("HOME not set");
        Path::new(&home).join("Library/Application Support/Bishop Engine/games")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").expect("APPDATA not set");
        Path::new(&appdata).join("Bishop Engine\\games")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let home = std::env::var("HOME").expect("HOME not set");
        Path::new(&home).join(".local/share/BishopEngine/games")
    }
}