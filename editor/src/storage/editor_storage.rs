// editor/src/storage/editor_storage.rs
use macroquad::prelude::*;
use uuid::Uuid;
use engine_core::{
    assets::asset_manager::AssetManager, constants::DEFAULT_TILE_SIZE, ecs::{
        component::{CurrentRoom, Player, Position}, 
        world_ecs::WorldEcs
    }, game::{game::Game, game_map::GameMap}, storage::path_utils::*, world::{
        room::Room,
        world::{World, WorldId, WorldMeta},
    }
};
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
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
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
    ron::de::from_str(&ron).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
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
) -> std::io::Result<()> {
    let old_game_dir = game_folder(&game.name);
    let new_game_dir = game_folder(&new_name);
    std::fs::rename(&old_game_dir, &new_game_dir)?;

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
) -> std::io::Result<()> {
    use std::path::Path;

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

    // Recursively copy the directory
    fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
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
    // Path of the running executable
    let exe = std::env::current_exe().ok()?;

    // Platform specific layout
    #[cfg(target_os = "macos")]
    {
        // …/Bishop Engine.app/Contents/MacOS/editor
        exe.parent() // MacOS/
            .and_then(|p| p.parent()) // Contents/
            .map(|p| p.join("Resources").join("game"))
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // …/Bishop Engine.exe  or  …/bishop-engine
        let game_dir = exe.parent()
            .expect("cannot locate bundled resources")
            .join("game");

        Some(game_dir)
    }
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