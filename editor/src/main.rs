// editor/src/main.rs
use crate::{
    editor::Editor, 
    global::*,
};
use engine_core::{constants::*, storage::path_utils::absolute_save_root};
use macroquad::prelude::*;

mod global;
mod controls;
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

fn window_conf() -> Conf {
    let width  = FIXED_WINDOW_WIDTH.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
    let height = FIXED_WINDOW_HEIGHT.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);

    Conf {
        window_title: "Bishop Engine".to_owned(),
        window_height: height,
        window_width: width,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    // Create folder structure if it doesn't exist
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

