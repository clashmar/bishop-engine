// game/src/main.rs
use crate::miniquad::conf::Platform;
use crate::rendering::render_system::RenderSystem;
use engine_core::storage::path_utils::resources_dir_from_exe;
use engine_core::assets::core_assets::load_rgba_resized;
use engine_core::camera::camera_manager::CameraManager;
use game_lib::scripting::lua_game_ctx::LuaGameCtx;
use game_lib::diagnostics::DiagnosticsOverlay;
use game_lib::frame_limiter::FrameLimiter;
use game_lib::game_state::GameState;
use macroquad::miniquad::conf::Icon;
use game_lib::engine::Engine;
use macroquad::prelude::*;
use std::cell::RefCell;
use engine_core::*;
use std::rc::Rc;
use mlua::Lua;
use std::env;
use std::fs;

fn window_conf() -> Conf {
    // Start with the default miniquad icon
    let mut icon = Some(Icon::miniquad_logo());

    if let Some(resources_dir) = resources_dir_from_exe() {
        let icon_path = resources_dir.join("Icon.png");

        // Read the file and make the icon
        if let Ok(png_bytes) = fs::read(&icon_path) {
            icon = Some(load_icon(&png_bytes));
        } else {
            onscreen_warn!("Could not read icon.")
        }
    }

    // Use the exe as the window title
    let window_title = env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "Game".to_string());

    Conf {
        window_title,
        fullscreen: true,
        window_resizable: true,
        icon,
        platform: Platform {
            swap_interval: Some(0), // VSync
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Helper that returns the icon.
fn load_icon(png_bytes: &[u8]) -> Icon {
    Icon {
        small: load_rgba_resized::<{ 16 * 16 * 4 }>(png_bytes, 16),
        medium: load_rgba_resized::<{ 32 * 32 * 4 }>(png_bytes, 32),
        big: load_rgba_resized::<{ 64 * 64 * 4 }>(png_bytes, 64),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    onscreen_info!("Initializing game.");

    // Pre-cache font to avoid black rectangle rendering bug
    engine_core::assets::core_assets::precache_font();

    let lua = Lua::new();
    let mut camera_manager = CameraManager::default();

    let game_state = Rc::new(RefCell::new(GameState::new(&lua, &mut camera_manager).await));
    let grid_size = game_state.borrow().game.current_world().grid_size;

    let ctx = LuaGameCtx { game_state: game_state.clone() };
    let _ = ctx.set_lua_game_ctx(&lua);

    let mut engine = Engine {
        game_state: game_state.clone(),
        lua,
        camera_manager,
        render_system: RenderSystem::with_grid_size(grid_size),
        diagnostics: DiagnosticsOverlay::new(),
        is_playtest: false,
    };

    engine.run().await;
}
