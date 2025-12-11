// game/src/playtest_main.rs
use engine_core::rendering::render_system::RenderSystem;
use engine_core::camera::camera_manager::CameraManager;
use game_lib::game_state::GameState;
use engine_core::world::room::Room;
use engine_core::game::game::Game;
use game_lib::engine::LuaGameCtx;
use engine_core::constants::*;
use game_lib::engine::Engine;
use macroquad::prelude::*;
use std::cell::RefCell;
use ron::de::from_str;
use std::{env, fs};
use std::rc::Rc;
use mlua::Lua;

/// The complete payload the editor writes for the play‑test binary.
#[derive(serde::Deserialize)]
struct PlaytestPayload {
    room: Room,
    game: Game,
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
        .expect("Could not read the temporary playtest file.");

    let PlaytestPayload {
        room,
        game,
    } = from_str(&payload_str).expect("Failed to deserialize playtest payload.");

    // TODO: Tidy up
    let lua = Lua::new();
    let mut camera_manager = CameraManager::default();

    let game_state = Rc::new(RefCell::new(GameState::for_room(room, game, &lua, &mut camera_manager).await));

    let ctx = LuaGameCtx { game_state: game_state.clone() };
    let _ = ctx.set_lua_game_ctx(&lua);

    let mut engine = Engine { 
        game_state: game_state.clone(), 
        lua, 
        camera_manager,
        render_system: RenderSystem::new(),
    };

    engine.run().await;
}