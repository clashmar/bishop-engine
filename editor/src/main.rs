use crate::editor::Editor;
use macroquad::prelude::*;

mod controls;
mod editor;
mod gui;
mod room;
mod storage;
mod tilemap;
mod world;

#[macroquad::main("Map Editor")]
async fn main() {
    let mut editor = Editor::new().await;

    loop {
        editor.update().await;
        editor.draw();
        next_frame().await
    }
}