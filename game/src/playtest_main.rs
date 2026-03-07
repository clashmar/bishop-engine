// game/src/playtest_main.rs
use engine_core::camera::camera_manager::CameraManager;
use game_lib::scripting::lua_ctx::register_lua_contexts;
use game_lib::game_instance::GameInstance;
use engine_core::prelude::*;
use game_lib::engine::Engine;
use bishop::prelude::*;
use bishop::BishopApp;
use std::cell::RefCell;
use ron::de::from_str;
use std::{env, fs};
use std::rc::Rc;
use mlua::Lua;

/// The complete payload the editor writes for the play-test binary.
#[derive(serde::Deserialize)]
struct PlaytestPayload {
    room: Room,
    game: Game,
}

/// Wrapper struct for running playtest via BishopApp.
struct PlaytestApp {
    payload_path: String,
    engine: Option<Engine>,
    ctx: Option<PlatformContext>,
}

impl PlaytestApp {
    fn new(payload_path: String) -> Self {
        Self {
            payload_path,
            engine: None,
            ctx: None,
        }
    }
}

impl BishopApp for PlaytestApp {
    async fn init(&mut self, ctx: PlatformContext) {
        // Store the context for later use
        self.ctx = Some(ctx.clone());

        set_engine_mode(
            EngineMode::Playtest
        );

        let payload_str = match fs::read_to_string(&self.payload_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Could not read the temporary playtest file: {e}");
                std::process::exit(1);
            }
        };

        let PlaytestPayload { room, game } = match from_str(&payload_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to deserialize playtest payload: {e}");
                std::process::exit(1);
            }
        };

        let lua = Lua::new();
        let mut camera_manager = CameraManager::default();
        let grid_size = game.current_world().grid_size;

        let game_instance = {
            let mut ctx_ref = ctx.borrow_mut();
            Rc::new(RefCell::new(
                GameInstance::for_room(
                    &mut *ctx_ref, 
                    room, game, 
                    &lua, 
                    &mut camera_manager, 
                    grid_size
                ).await
            ))
        };

        let _ = register_lua_contexts(&lua, game_instance.clone(), ctx.clone());

        self.engine = Some(Engine::new(
            game_instance.clone(),
            ctx.clone(),
            lua,
            camera_manager,
            grid_size,
            true,
        ));
    }

    async fn frame(&mut self, ctx: PlatformContext) {
        if let Some(engine) = &mut self.engine {
            engine.frame(ctx).await;
        }
    }
}

fn main() -> Result<(), RunError> {
    // Load the temporary file written by the editor
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <playtest_payload.ron>", args[0]);
        std::process::exit(1);
    }

    let payload_path = args[1].clone();

    let width = FIXED_WINDOW_WIDTH.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
    let height = FIXED_WINDOW_HEIGHT.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);

    let config = WindowConfig::new("Playtest")
        .with_size(width as u32, height as u32)
        .with_resizable(true);

    let app = PlaytestApp::new(payload_path);
    run_backend(config, app)
}
