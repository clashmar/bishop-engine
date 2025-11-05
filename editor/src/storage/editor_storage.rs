// editor/src/storage/editor_storage.rs
use macroquad::prelude::*;
use uuid::Uuid;
use engine_core::{
    assets::asset_manager::AssetManager, constants::DEFAULT_TILE_SIZE, ecs::{
        component::{CurrentRoom, Player, Position}, 
        world_ecs::WorldEcs
    }, game::{game::Game, game_map::GameMap}, storage::path_utils::*, world::{
        room::Room,
        world::{World, WorldId},
    }
};
use std::{
    fs, io, time::SystemTime
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
        starting_room: Some(room_id),
        starting_position: Some(starting_position),
        map_position: Vec2::ZERO,
    };

    let _player = world.world_ecs
        .create_entity()
        .with(Player)
        .with(Position { position: starting_position })
        .with(CurrentRoom(room_id))
        .finish();

    world
}

/// Prompt the user for a string input using Macroquad’s UI loop.
pub async fn prompt_user_input() -> Option<String> {
    // Consume any remaining chars in the input queue.
    while get_char_pressed().is_some() {}

    let mut input = String::new();

    loop {
        clear_background(BLACK);

        // Gather newly pressed characters.
        while let Some(c) = get_char_pressed() {
            if c.is_alphanumeric()
                || c.is_ascii_whitespace()
                || c.is_ascii_punctuation()
            {
                input.push(c);
            }
        }

        // Backspace handling.
        if is_key_pressed(KeyCode::Backspace) {
            input.pop();
        }

        // Escape cancels the prompt.
        if is_key_pressed(KeyCode::Escape) {
            return None;
        }

        // Enter confirms the input (if not empty).
        if is_key_pressed(KeyCode::Enter) && !input.trim().is_empty() {
            return Some(input);
        }

        // Draw the prompt box.
        let text = format!("Enter game name: {}", input);
        draw_rectangle(100., 100., 600., 100., DARKGRAY);
        draw_text(&text, 120., 160., 30., WHITE);

        next_frame().await;
    }
}