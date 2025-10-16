// game/src/playtest_main.rs
use std::{env, fs};
use engine_core::{
    constants::{
        world_virtual_height, 
        world_virtual_width
    }, 
    world::{
        room::Room, 
        world::World
    }
};
use game_lib::game::GameState;
use macroquad::prelude::*;
use ron::de::from_str;

/// The complete payload the editor writes for the play‑test binary.
#[derive(serde::Deserialize)]
struct PlaytestPayload {
    room: Room,
    world: World,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Playtest".to_owned(),
        window_width: world_virtual_width() as i32,
        window_height: world_virtual_height() as i32,
        fullscreen: true,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Load the temporary file written by the editor
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <playtest_payload.ron>", args[0]);
        std::process::exit(1);
    }

    let payload_path = &args[1];
    let payload_str = fs::read_to_string(payload_path)
        .expect("could not read the temporary play‑test file");

    let PlaytestPayload {
        room,
        world,
    } = from_str(&payload_str).expect("Failed to deserialize play‑test payload.");

    let mut game = GameState::for_room(room, world).await;

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}