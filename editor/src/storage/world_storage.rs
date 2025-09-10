// editor/src/storage/world_storage.rs
use macroquad::prelude::*;
use uuid::Uuid;
use engine_core::{
    constants::WORLD_SAVE_FOLDER, ecs::world_ecs::WorldEcs, world::{
        room::{Room, RoomMetadata},
        world::World,
    }
};
use std::{
    collections::HashMap, fs, io, path::{Path}, time::SystemTime
};
use crate::{
    storage::world_storage, 
    tilemap::tile_palette::TilePalette
};

type WorldIndex = HashMap<Uuid, String>;

/// Create a fresh world with a single default room.
pub fn create_new_world(name: String) -> World {
    let id = Uuid::new_v4();
    let ecs = WorldEcs::default();
    let first_room_metadata = RoomMetadata::default();
    let room_id = first_room_metadata.id;
    let first_room = Room::default();

    let world = World {
        id,
        name: name.clone(),
        world_ecs: ecs,
        rooms_metadata: vec![first_room_metadata],
        starting_room: Some(room_id),
        starting_position: Some(vec2(1.0, 1.0)),
    };

    // Save the world.
    if let Err(e) = world_storage::save_world(&world) {
        eprintln!("Could not save the initial room: {e}");
    }

    // Save the room.
    if let Err(e) = world_storage::save_room(
        &world.id,             
        room_id,             
        &first_room,        
    ) {
        eprintln!("Could not save the initial room: {e}");
    }

    match load_index() {
        Ok(mut idx) => {
            idx.insert(world.id, name);
            if let Err(e) = save_index(&idx) {
                eprintln!("Failed to update world index: {e}");
            }
        }
        Err(e) => {
            eprintln!("Could not load world index (will create a new one): {e}");
            // If the index file does not exist we create a fresh one.
            let mut idx = HashMap::new();
            idx.insert(world.id, name);
            if let Err(e) = save_index(&idx) {
                eprintln!("Failed to write new world index: {e}");
            }
        }
    }
    world
}

/// Write the `World`, including room metadata, to WORLD_SAVE_FOLDER.
pub fn save_world(world: &World) -> io::Result<()> {
    let pretty = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let ron_string = ron::ser::to_string_pretty(world, pretty)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Folder is the UUID, file is always "world.ron"
    let dir_path = Path::new(WORLD_SAVE_FOLDER).join(world.id.to_string());
    let file_path = dir_path.join("world.ron");

    fs::create_dir_all(&dir_path)?;
    fs::write(file_path, ron_string)
}

/// Load a world from its *.ron* file.
pub fn load_world_by_id(id: &Uuid) -> io::Result<World> {
    let path = Path::new(WORLD_SAVE_FOLDER)
        .join(id.to_string())
        .join("world.ron");
    let ron_string = fs::read_to_string(path)?;
    ron::from_str(&ron_string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Save a single room. Called automatically when the user leaves the room editor.
pub fn save_room(world_id: &Uuid, id: Uuid, room: &Room) -> io::Result<()> {
    let base = Path::new(WORLD_SAVE_FOLDER).join(world_id.to_string());
    let rooms_dir = base.join("rooms");
    fs::create_dir_all(&rooms_dir)?; // create on first save if missing

    let room_path = rooms_dir.join(format!("{}.ron", id));
    let ron_string = ron::ser::to_string_pretty(room, ron::ser::PrettyConfig::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(room_path, ron_string)
}

/// Load a single room by its UUID. Called when the user opens a room.
pub fn load_room(world_id: &Uuid, room_id: Uuid) -> io::Result<Room> {
    let base = Path::new(WORLD_SAVE_FOLDER).join(world_id.to_string());
    let room_path = base.join("rooms").join(format!("{}.ron", room_id));
    let data = fs::read_to_string(room_path)?;
    ron::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Delete a single room by its UUID.
pub fn delete_room_file(world_id: &Uuid, room_id: Uuid) -> io::Result<()> {
    let path = Path::new(WORLD_SAVE_FOLDER)
        .join(world_id.to_string())
        .join("rooms")
        .join(format!("{}.ron", room_id));
    std::fs::remove_file(path)
}

pub fn load_index() -> io::Result<WorldIndex> {
    let path = Path::new(WORLD_SAVE_FOLDER).join("index.ron");
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let s = fs::read_to_string(path)?;
    ron::from_str(&s).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn save_index(idx: &WorldIndex) -> io::Result<()> {
    let s = ron::ser::to_string_pretty(idx, ron::ser::PrettyConfig::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let path = Path::new(WORLD_SAVE_FOLDER).join("index.ron");
    fs::write(path, s)
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
        let text = format!("Enter world name: {}", input);
        draw_rectangle(100., 100., 600., 100., DARKGRAY);
        draw_text(&text, 120., 160., 30., WHITE);

        next_frame().await;
    }
}

/// Return the name of the most‑recently‑modified world directory,
/// or `None` if the folder does not exist or contains no sub‑directories.
pub fn most_recent_world_id() -> Option<Uuid> {
    let root = Path::new(WORLD_SAVE_FOLDER);
    let mut best: Option<(Uuid, SystemTime)> = None;

    for entry in fs::read_dir(root).ok()? {
        let entry = entry.ok()?;                     
        if !entry.path().is_dir() { continue; }

        let name = entry
            .file_name()               
            .to_string_lossy()         
            .into_owned();              

        if let Ok(uuid) = Uuid::parse_str(&name) {
            let mod_time = entry.metadata().ok()?.modified().ok()?;
            match best {
                None => best = Some((uuid, mod_time)),
                Some((_, t)) if mod_time > t => best = Some((uuid, mod_time)),
                _ => {}
            }
        }
    }

    best.map(|(id, _)| id)
}

/// Write the palette to `<world_dir>/palette.ron`
pub fn save_palette(palette: &TilePalette, world_id: &Uuid) -> io::Result<()> {
    let dir = Path::new(WORLD_SAVE_FOLDER).join(world_id.to_string());
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("palette.ron");
    let ron = ron::ser::to_string(palette)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    std::fs::write(path, ron)
}

/// Load a palette.  If the file does not exist return a default palette.
pub fn load_palette(world_id: &Uuid) -> io::Result<TilePalette> {
    let path = Path::new(WORLD_SAVE_FOLDER)
        .join(world_id.to_string())
        .join("palette.ron");

    if !path.exists() {
        return Ok(TilePalette::new(
            vec2(10.0, 10.0),
            32.0,             
            2,               
            2,                
        ));
    }

    let ron = std::fs::read_to_string(path)?;
    ron::de::from_str(&ron).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}