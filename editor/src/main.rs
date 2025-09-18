// editor/src/main.rs
use crate::editor::Editor;
use engine_core::constants::*;
use macroquad::prelude::*;

mod controls;
mod editor;
mod gui;
mod room;
mod storage;
mod tilemap;
mod world;
mod camera_controller;
mod canvas;
mod playtest;

/// Macroquad configuration – called once before `main`.
/// We keep the window size equal to the virtual resolution and
/// prevent the user from changing the aspect ratio.
pub fn conf() -> Conf {
    Conf {
        window_title: "World Editor".to_owned(),
        window_width: WORLD_VIRTUAL_WIDTH as i32,
        window_height: WORLD_VIRTUAL_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main("World Editor")]
async fn main() -> std::io::Result<()> {
    let mut editor = Editor::new().await?;

    loop {
        editor.update().await;
        editor.draw();
        next_frame().await
    }
}

