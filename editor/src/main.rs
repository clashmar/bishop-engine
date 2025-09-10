// editor/src/main.rs
use crate::editor::Editor;
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
mod entities;

#[macroquad::main("World Editor")]
async fn main() -> std::io::Result<()> {
    let mut editor = Editor::new().await?;

    loop {
        editor.update().await;
        editor.draw();
        next_frame().await
    }
}