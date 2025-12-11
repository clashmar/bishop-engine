// editor/src/storage/editor_storage.rs
#![allow(unused)]
use crate::scripting::script_manager::ScriptManager;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use engine_core::input::input_snapshot::InputSnapshot;
use engine_core::scripting::script_manager;
use engine_core::storage::editor_config::app_dir;
use crate::tilemap::tile_palette::TilePalette;
use std::io::Write;
use std::rc::Rc;
use std::time::SystemTime;
use std::io;
use std::fs;
use std::path::PathBuf;
use engine_core::*;
use engine_core::world::room::Room;
use engine_core::ecs::component::*;
use engine_core::ecs::world_ecs::*;
use engine_core::world::world::*;
use engine_core::game::game_map::GameMap;
use engine_core::constants::*;
use engine_core::assets::asset_manager::*;
use engine_core::storage::path_utils::*;
use engine_core::game::game::*;
use macroquad::prelude::*;
use uuid::Uuid;
use std::io::Error;
use std::io::ErrorKind;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Create a brand‑new game with a single empty world.
pub async fn create_new_game(name: String) -> Game {
    onscreen_debug!("Creating new game.");

    // Ensure the folder structure exists.
    create_game_folders(&name);

    // Build the game
    let world = create_new_world();
    let current_id = world.id;

    let asset_manager = AssetManager::new(name.clone()).await;
    let script_manager = ScriptManager::new(name.clone()).await;

    let game = Game {
        save_version: 1,
        id: Uuid::new_v4(),
        name,
        worlds: vec![world],
        asset_manager,
        script_manager,
        current_world_id: current_id,
        tile_size: DEFAULT_TILE_SIZE,
        game_map: GameMap::default(),
    };

    // Save the game.
    if let Err(e) = save_game(&game) {
        onscreen_error!("Could not save the new game: {e}");
    }

    game
}

fn create_game_folders(name: &String) {
    let folders: [(PathBuf, &str); 5] = [
        (resources_folder(&name), RESOURCES_FOLDER),
        (assets_folder(&name), ASSETS_FOLDER),
        (scripts_folder(&name), SCRIPTS_FOLDER),
        (windows_folder(&name), WINDOWS_FOLDER),
        (mac_os_folder(&name), MAC_OS_FOLDER),
    ];

    for (path, folder) in folders {
        if let Err(e) = fs::create_dir_all(&path) {
            onscreen_error!("Could not create {folder} folder '{}': {e}", path.display());
        }
    }
}

/// Save a `Game` and all its contents.
pub fn save_game(game: &Game) -> io::Result<()> {
    let pretty = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    
    let ron_string = ron::ser::to_string_pretty(game, pretty)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let resources_folder = resources_folder(&game.name);
    let file_path = resources_folder.join(GAME_RON);
    
    fs::create_dir_all(&resources_folder)?;
    onscreen_info!("{}", file_path.display());
    fs::write(file_path, ron_string)
}

/// Load a `Game` from the folder that matches the supplied name.
pub async fn load_game_by_name(name: &str) -> io::Result<Game> {
    let path = resources_folder(name).join(GAME_RON);
    onscreen_debug!("Loading game from .ron: {}.", path.display());

    // Try to read the file
    let ron_string = match fs::read_to_string(&path) {
        Ok(s) => s,
        // File not found
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            return Ok(create_new_game(name.to_string()).await);
        }
        // Other I/O errors
        Err(e) => return Err(e),
    };

    // Parse the RON
    match ron::from_str::<Game>(&ron_string) {
        Ok(mut game) => {
            game.initialize().await;
            Ok(game)
        },
        // Corrupt file
        Err(_) => Ok(create_new_game(name.to_string()).await),
    }
}

/// Return the name of the most recently modified game folder.
pub fn most_recent_game_name() -> Option<String> {
    let root = absolute_save_root();  
    let mut best: Option<(String, SystemTime)> = None;
    
    for entry in fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Ok(mod_time) = entry.metadata().ok()?.modified() {
            match best {
                None => best = Some((name, mod_time)),
                Some((_, t)) if mod_time > t => best = Some((name, mod_time)),
                _ => {}
            }
        }
    }
    best.map(|(name, _)| name)
}

/// Save the palette for the game.
pub fn save_palette(palette: &TilePalette, game_name: &str) -> io::Result<()> {
    let dir = game_folder(game_name);
    fs::create_dir_all(&dir)?;
    let path = dir.join("palette.ron");
    let ron = ron::ser::to_string(palette)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, ron)
}

/// Load the palette from the game folder.
pub fn load_palette(game_name: &str) -> io::Result<TilePalette> {
    let path = game_folder(game_name).join("palette.ron");
    if !path.exists() {
        return Ok(TilePalette::new());
    }
    let ron = fs::read_to_string(path)?;
    ron::de::from_str(&ron).map_err(|e| Error::new(ErrorKind::Other, e))
}

/// Create a fresh world with a single default room.
pub fn create_new_world() -> World {
    let id = WorldId(Uuid::new_v4());
    let name = "new".to_string();
    let mut world_ecs = WorldEcs::default();
    let first_room = Room::default(&mut world_ecs);
    let room_id = first_room.id;
    let starting_position = vec2(1.0, 1.0);

    let mut world = World {
        id,
        name: name.clone(),
        world_ecs,
        rooms: vec![first_room],
        current_room_id: None,
        starting_room_id: Some(room_id),
        starting_position: Some(starting_position),
        meta: WorldMeta::default()
    };

    let _player = world.world_ecs
        .create_entity()
        .with(Player)
        .with(Position { position: starting_position })
        .with(CurrentRoom(room_id))
        .finish();

    world
}

/// Rename a game folder and assets. 
/// Returns `Ok(())` on success or an `io::Error` on failure.
pub fn rename_game(
    game: &mut Game,
    new_name: &str,
) -> io::Result<()> {
    let old_game_dir = game_folder(&game.name);
    let new_game_dir = game_folder(new_name);
    fs::rename(&old_game_dir, &new_game_dir)?;

    // Asset manager uses the game name to find the assets folder
    game.asset_manager.game_name = new_name.to_owned();
    game.name = new_name.to_owned();
    Ok(())
}

/// Save a copy of the current game in a newly named folder. 
/// Returns `Ok(())` on success or an `io::Error` on failure.
pub fn save_as(
    game: &mut Game,
    new_name: &str,
) -> io::Result<()> {
    // Determine paths
    let old_game_dir = game_folder(&game.name);
    let new_game_dir = game_folder(new_name);
    let old_assets_dir = assets_folder(&game.name);
    let new_assets_dir = assets_folder(new_name);

    // Guard against overwriting an existing game
    if new_game_dir.exists() || new_assets_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("A game called \"{new_name}\" already exists"),
        ));
    }

    // Copy the game and assets folder
    copy_dir_recursive(&old_game_dir, &new_game_dir)?;
    copy_dir_recursive(&old_assets_dir, &new_assets_dir)?;

    // Update the game and assets manager
    game.name = new_name.to_owned();
    game.asset_manager.game_name = new_name.to_owned();

    Ok(())
}

/// Writes an embedded slice of bytes to the system app directory and returns the path or error.
pub fn write_to_app_dir(filename: &str, embedded: &[u8]) -> io::Result<PathBuf> {
    let mut path = app_dir();
    fs::create_dir_all(&path)?;

    path.push(filename);

    let mut file = fs::File::create(&path)?;
    file.write_all(embedded)?;

    #[cfg(target_os = "macos")]
    {
        // Set executable permissions
        onscreen_debug!("Writing binary permissions.");
        let mut permissions = fs::metadata(&path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)?;
    }

    Ok(path)
}

/// Find all game folders in `games/`.
pub fn list_game_folders() -> io::Result<Vec<PathBuf>> {
    let root = match cfg!(debug_assertions) {
        true => absolute_save_root(),
        false => absolute_save_root().join(GAME_SAVE_ROOT),
    };

    let mut folders = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join(GAME_RON).exists() {
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

/// Returns a Vec of all game names in the absolute save root.
pub fn list_game_names() -> Vec<String> {
    std::fs::read_dir(absolute_save_root())
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}