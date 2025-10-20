// game/src/playtest_main.rs
use std::{env, fs};
use engine_core::{
    constants::*, world::{
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
    let width  = FIXED_WINDOW_WIDTH.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
    let height = FIXED_WINDOW_HEIGHT.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);
    
    Conf {
        window_title: "Playtest".to_owned(),
        window_width: width,
        window_height: height,
        fullscreen: false,
        window_resizable: true,
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
        .expect("could not read the temporary playtest file");

    let PlaytestPayload {
        room,
        world,
    } = from_str(&payload_str).expect("Failed to deserialize playtest payload.");

    let mut game = GameState::for_room(room, world).await;
    let mut accumulator = 0.0_f32;

    loop {
        // Time elapsed since last frame
        let frame_dt = get_frame_time();
        accumulator += frame_dt;

        // Clamp the backlog
        if accumulator > MAX_ACCUM {
            accumulator = MAX_ACCUM;
        }

        // Fixed‑step physics
        while accumulator >= FIXED_DT {
            game.fixed_update(FIXED_DT);
            accumulator -= FIXED_DT;
        }

        // Per frame async work
        game.update_async(frame_dt).await;

        // Interpolation factor for rendering
        let alpha = accumulator / FIXED_DT;
        game.render(alpha);
        next_frame().await;
    }
}