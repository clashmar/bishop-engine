use core::{
    constants::WORLD_SAVE_FOLDER,
    world::{
        room::{Room, RoomMetadata},
        world::World,
    },
};
use macroquad::prelude::*;
use uuid::Uuid;
use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::storage::world_storage;

/// Create a fresh world with a single default room.
pub fn create_new_world(name: String) -> World {
    let first_room_metadata = RoomMetadata::default();
    let room_id = first_room_metadata.id;
    let first_room = Room::default();

    let world = World {
        name,
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
        &world.name,             
        room_id,             
        &first_room,        
    ) {
        eprintln!("Could not save the initial room: {e}");
    }
    world
}

/// Write the `World`, including room metadata, to *WORLD_SAVE_FOLDER*.
pub fn save_world(world: &World) -> io::Result<()> {
    let pretty = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let ron_string = ron::ser::to_string_pretty(world, pretty)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Build the directory and file path.
    let dir_path = Path::new(WORLD_SAVE_FOLDER).join(&world.name);
    let file_path = dir_path.join(format!("{}.ron", &world.name));

    // Ensure the directory exists.
    fs::create_dir_all(&dir_path)?;

    // Write the serialized data.
    fs::write(&file_path, ron_string)
}

/// Load a world from its *.ron* file.
pub fn load_world(world_name: &str) -> io::Result<World> {
    let path: PathBuf = Path::new(WORLD_SAVE_FOLDER)
        .join(world_name)
        .join(format!("{}.ron", world_name));

    let ron_string = fs::read_to_string(&path)?;
    ron::from_str(&ron_string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Save a single room. Called automatically when the user leaves the room editor.
pub fn save_room(world_name: &str, id: Uuid, room: &Room) -> io::Result<()> {
    let base = Path::new(WORLD_SAVE_FOLDER).join(world_name);
    let rooms_dir = base.join("rooms");
    fs::create_dir_all(&rooms_dir)?; // create on first save if missing

    let room_path = rooms_dir.join(format!("{}.ron", id));
    let ron_string = ron::ser::to_string_pretty(room, ron::ser::PrettyConfig::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(room_path, ron_string)
}

/// Load a single room by its UUID. Called when the user opens a room.
pub fn load_room(world_name: &str, id: Uuid) -> io::Result<Room> {
    let base = Path::new(WORLD_SAVE_FOLDER).join(world_name);
    let room_path = base.join("rooms").join(format!("{}.ron", id));
    let data = fs::read_to_string(room_path)?;
    ron::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Delete a single room by its UUID.
pub fn delete_room_file(world_name: &str, id: Uuid) -> io::Result<()> {
    let path = Path::new(WORLD_SAVE_FOLDER)
        .join(world_name)
        .join("rooms")
        .join(format!("{}.ron", id));
    std::fs::remove_file(path)
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
pub fn most_recent_world() -> Option<String> {
    let root = Path::new(WORLD_SAVE_FOLDER);
    if !root.is_dir() {
        return None;
    }

    let mut dirs: Vec<(fs::DirEntry, SystemTime)> = Vec::new();

    // Iterate over entries synchronously.
    for entry_res in fs::read_dir(root).ok()? {
        let entry = entry_res.ok()?;
        if entry.path().is_dir() {
            let mod_time = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            dirs.push((entry, mod_time));
        }
    }

    // Sort newest‑first.
    dirs.sort_by_key(|(_, time)| *time);
    dirs.reverse();

    dirs.first()
        .and_then(|(entry, _)| entry.file_name().to_str().map(|s| s.to_owned()))
}