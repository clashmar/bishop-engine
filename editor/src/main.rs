// editor/src/main.rs

// Tells windows if it's a console app or not (console is useful in debug)
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::editor_assets::editor_assets::*;
use crate::editor_global::*;
use crate::editor::Editor;
use engine_core::logging::logging::init_file_logger;
use engine_core::ui::widgets::*;
use engine_core::*;
use engine_core::storage::path_utils::*;
use engine_core::{constants::*, storage::path_utils::absolute_save_root};
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;
use bishop::prelude::{PlatformContext, run};
use bishop::{BishopApp, BishopContext};

mod editor_global;
mod editor;
mod gui;
mod room;
mod storage;
mod tilemap;
mod world;
mod editor_camera_controller;
mod canvas;
mod playtest;
mod commands;
mod game;
mod editor_assets;
mod editor_actions;

/// Wrapper struct for running the editor via BishopApp.
struct EditorApp {
    current_window_size: (u32, u32),
}

impl EditorApp {
    fn new() -> Self {
        Self {
            current_window_size: (0, 0),
        }
    }
}

impl BishopApp for EditorApp {
    async fn frame(&mut self, ctx: &mut impl BishopContext) {
        let cur_screen = (ctx.screen_width() as u32, ctx.screen_height() as u32);
        if cur_screen != self.current_window_size {
            with_editor(|editor| editor.render_system.resize(cur_screen.0, cur_screen.1));
            self.current_window_size = cur_screen;
        }

        widgets_frame_start(ctx);

        with_editor_async(ctx, |editor, ctx| Box::pin(editor.update(ctx))).await;
        with_editor_async(ctx, |editor, ctx| Box::pin(editor.draw(ctx))).await;

        widgets_frame_end(ctx);

        apply_pending_commands();
    }
}

fn window_conf() -> Conf {
    let window_width  = FIXED_WINDOW_WIDTH.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
    let window_height = FIXED_WINDOW_HEIGHT.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);

    let icon: Icon = Icon {
        small: *ICON_SMALL,
        medium: *ICON_MEDIUM,
        big: *ICON_BIG,
    };

    Conf {
        window_title: "Bishop Engine".to_owned(),
        window_height,
        window_width,
        fullscreen: false,
        window_resizable: true,
        icon: Some(icon),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    onscreen_info!("Starting editor.");

    // Initialize logging
    init_file_logger();

    // Pre-cache font to avoid macroquad text bug
    engine_core::assets::core_assets::precache_font();

    if !ensure_save_root().await {
        // User cancelled
        onscreen_warn!("No save root selected. Exiting.");
        std::process::exit(0);
    }

    let games_path = absolute_save_root();
    std::fs::create_dir_all(&games_path)?;
    
    let editor = Editor::new().await?;

    // This allows global access to services
    set_editor(editor);

    let mut app = EditorApp::new();
    let mut ctx = PlatformContext::new();
    run(&mut app, &mut ctx).await;

    Ok(())
}