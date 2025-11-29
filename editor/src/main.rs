// editor/src/main.rs
#![windows_subsystem = "windows"] 

use crate::editor_assets::editor_assets::*;
use crate::global::*;
use crate::editor::Editor;
use engine_core::logging::logging::init_file_logger;
use engine_core::storage::path_utils::*;
use engine_core::{constants::*, storage::path_utils::absolute_save_root};
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::*;

mod global;
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
    // Initialize logging
    init_file_logger();    

    if !ensure_save_root().await {
        // User cancelled
        println!("No save root selected. Exiting.");
        std::process::exit(0);
    }

    let games_path = absolute_save_root();
    std::fs::create_dir_all(&games_path)?;
    
    let editor = Editor::new().await?;

    // This allows the command manager global access
    set_editor(editor);

    let mut current_window_size = (screen_width() as u32, screen_height() as u32);

    loop {
        // Update the render targets with the current window size
        let cur_screen = (screen_width() as u32, screen_height() as u32);
        if cur_screen != current_window_size {
            with_editor(|editor| 
                Box::pin(editor.render_system.resize(cur_screen.0, cur_screen.1))
            );
            current_window_size = cur_screen;
        }

        with_editor_async(|editor| Box::pin(editor.update())).await;
    
        with_editor_async(|editor| Box::pin(editor.draw())).await;
        
        apply_pending_commands();
        
        next_frame().await
    }
}