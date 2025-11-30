// game/src/main.rs
use std::fs;
use engine_core::assets::core_assets::load_rgba_resized;
use engine_core::*;
use engine_core::storage::path_utils::resources_dir_from_exe;
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use game_lib::game::GameState;

fn window_conf() -> Conf {
    // Start with the default miniquad icon
    let mut icon = Some(Icon::miniquad_logo());

    // Try to set the user defined icon TODO: work it out for windows
    if let Some(resources_dir) = resources_dir_from_exe() {
        let icon_path = resources_dir.join("Icon.png");

        // Read the file and make the icon
        if let Ok(png_bytes) = fs::read(&icon_path) {
            icon = Some(load_icon(&png_bytes));
        } else {
            onscreen_warn!("Could not read icon")
        }
    }

    // TODO Create and get game config for conf:
    Conf {
        window_title: "Zelda".to_owned(),
        fullscreen: true,
        window_resizable: true,
        icon,
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
    let mut game = GameState::new().await;
    game.run_game_loop().await;
}
