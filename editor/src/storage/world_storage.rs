use core::{constants::WORLD_SAVE_FOLDER, world::{room::RoomMetadata, world::World}};
use macroquad::{prelude::*};
use std::{fs, path::Path};

pub fn create_new_world(name: String) -> World {
    let first_room_metadata = RoomMetadata::default();
    World { 
        name,
        rooms_metadata: vec![first_room_metadata],
        starting_room: Some(0),                  
        starting_position: Some(vec2(1.0, 1.0)),
    }
}

pub async fn save_world(world: &World) {
    let pretty = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);

    let path = format!("{}/{}.ron", WORLD_SAVE_FOLDER, world.name);
    let ron_string = ron::ser::to_string_pretty(world, pretty).unwrap();
    fs::write(&path, ron_string).unwrap();
}

pub async fn prompt_user() -> Option<String>{
    while get_char_pressed().is_some() {} // Reset char input queue
    let mut input = String::new();
    let finished = false;

    while !finished {
        clear_background(BLACK);

        while let Some(c) = get_char_pressed() {
            if c.is_alphanumeric() || 
            c.is_ascii_whitespace() || 
            c.is_ascii_punctuation() {
                input.push(c);
            }
        }

        // Handle backspace
        if is_key_pressed(KeyCode::Backspace) {
            input.pop();
        }

        // Handle backspace
        if is_key_pressed(KeyCode::Escape) {
            return None
        }

        // Finish when Enter pressed
        if is_key_pressed(KeyCode::Enter) && !input.trim().is_empty() {
            return Some(input);
        }

        // Draw prompt box
        let text = format!("Enter world name: {}", input);
        draw_rectangle(100., 100., 600., 100., DARKGRAY);
        draw_text(&text, 120., 160., 30., WHITE);

        next_frame().await;
    }
    Some(input)
}

pub fn load_world(filename: &str) -> World {
    let path = format!("{}/{}.ron", WORLD_SAVE_FOLDER, filename);
    let ron_string = fs::read_to_string(&path).unwrap();
    ron::from_str(&ron_string).unwrap()
}

pub fn most_recent_world() -> Option<String> {
    let folder = Path::new(WORLD_SAVE_FOLDER);
    if !folder.exists() {
        return None;
    }

    let mut entries: Vec<_> = fs::read_dir(folder)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "ron").unwrap_or(false))
        .collect();

    // Sort by modified time, descending
    entries.sort_by_key(|e| {
        e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    entries.reverse();

    // Return the filename without extension
    entries.first().map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
}