// editor/src/main.rs
use crate::{
    editor::Editor, 
    global::*
};
use engine_core::constants::*;
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
        fullscreen: true,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let editor = Editor::new().await?;

    // This allows the command manager global access
    set_editor(editor);

    loop {
        with_editor_async(|editor| Box::pin(editor.update())).await;
    
        with_editor(|ed| {
            ed.draw();
        });

        apply_pending_commands();
        
        next_frame().await
    }
}

