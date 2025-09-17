// game/src/playtest_main.rs
mod camera;
mod game;
mod modes;

use std::{env, fs};
use engine_core::
    world::{
        room::Room, 
        world::World
    };
use macroquad::prelude::*;
use ron::de::from_str;
use crate::game::GameState;

/// The complete payload the editor writes for the play‑test binary.
#[derive(serde::Deserialize)]
struct PlaytestPayload {
    room: Room,
    world: World,
}

#[macroquad::main("Play‑test")]
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