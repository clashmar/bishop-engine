// editor/src/storage/editor_storage.rs
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
use std::{
    fs, io, path::PathBuf, time::SystemTime
};
use crate::{
    storage::{editor_storage}, 
    tilemap::tile_palette::TilePalette
};

/// Create a brand‑new game with a single empty world.
pub async fn create_new_game(name: String) -> Game {
    // Ensure the folder structure exists.
    let assets = assets_folder(&name);

    // Does nothing if the folder already exists
    if let Err(e) = fs::create_dir_all(&assets) {
        eprintln!("Could not create assets folder '{}': {e}", assets.display());
    }

    // Build the game
    let world = create_new_world();
    let current_id = world.id;

    let asset_manager = AssetManager::new(name.clone()).await;

    let game = Game {
        save_version: 1,
        id: Uuid::new_v4(),
        name,
        worlds: vec![world],
        asset_manager,
        current_world_id: current_id,
        tile_size: DEFAULT_TILE_SIZE,
        game_map: GameMap::default(),
    };

    // Save the game.
    if let Err(e) = editor_storage::save_game(&game) {
        eprintln!("Could not save the new game: {e}");
    }

    game
}

/// Save a `Game` and all its contents.
pub fn save_game(game: &Game) -> io::Result<()> {
    let pretty = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let ron_string = ron::ser::to_string_pretty(game, pretty)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let dir = game_folder(&game.name);
    let file_path = dir.join("game.ron");
    fs::create_dir_all(&dir)?;
    fs::write(file_path, ron_string)
}

/// Load a `Game` from the folder that matches the supplied name.
pub async fn load_game_by_name(name: &str) -> io::Result<Game> {
    let path = game_folder(name).join("game.ron");
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
    let new_game_dir = game_folder(&new_name);
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

/// Returns the absolute path to the bundled game binaries.
pub fn game_binary_dir() -> Option<PathBuf> {
    if let Some(resources_dir) = resources_dir() {
        return Some(resources_dir.join("binaries"));
    }
    None
}

/// Returns the absolute path to the bundled platform app templates.
fn templates_dir() -> Option<PathBuf> {
    if let Some(resources_dir) = resources_dir() {
        return Some(resources_dir.join("templates"));
    }
    None
}

/// Exports the game to the chosen folder on all platforms.
pub async fn export_game(game: &Game) -> io::Result<PathBuf> {
    let dest_root = rfd::FileDialog::new()
        .set_title("Select destination folder for export:")
        .pick_folder()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,  
                "No destination folder was selected.")
        })?;

    // TODO: Handle by platform

    // TODO: Check for duplicates

    let bundle_path = dest_root.join(format!("{}.app", game.name));

    // Copy template structure
    let template_dir = templates_dir()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,  
                "Could not find templates.",)
        })?;

    let template_path = template_dir.join("template.app");
    copy_dir_recursive(&template_path, &bundle_path)?;

    // Copy game binary
    let macos_dir = bundle_path
        .join("Contents")
        .join("MacOS");

    // Make sure this file exists
    fs::create_dir_all(&macos_dir)?;

    let game_binary_dir = game_binary_dir()
    .ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,  
            "Could not find game binaries.",)
        })?;

    let src_binary = game_binary_dir.join("game");
    let target_binary = macos_dir.join(&game.name);
    fs::copy(src_binary, &target_binary)?;

    // Copy assets
    let src_assets = assets_folder(&game.name);

    let target_assets = bundle_path
        .join("Contents")
        .join("Resources")
        .join("assets");
    
    copy_dir_recursive(&src_assets, &target_assets)?;

    // Copy the game.ron
    let src_ron = game_folder(&game.name)
        .join("game.ron");

    let target_ron = bundle_path
        .join("Contents")
        .join("Resources")
        .join("game.ron");

    fs::copy(src_ron, target_ron)?;

    // Create Info.plist TODO: this does not work
    let target_plist = bundle_path.join("Contents").join("Info.plist");
    let mut plist = fs::read_to_string(&target_plist)?;
    plist = plist
        .replace("__BUNDLE_NAME__", &game.name)
        .replace("__BUNDLE_IDENTIFIER__", &format!("com.bishop.{}", game.name.to_lowercase()))
        .replace("__BUNDLE_VERSION__", "0.1.0");
    fs::write(&target_plist, plist)?;

    // Copy app icons
    let src_icons = game_folder(&game.name)
        .join("Icon.icns");

    let target_icons = bundle_path
        .join("Contents")
        .join("Resources")
        .join("Icon.icns");

    fs::copy(src_icons, target_icons)?;

    Ok(bundle_path)
}

/// Recursively copy the directory. TODO: Research why it sometimes ignores empty dirs.
fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
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
        onscreen_error!("{:?}", entry.file_name());
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

pub fn list_game_names() -> Vec<String> {
    std::fs::read_dir(absolute_save_root())
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}