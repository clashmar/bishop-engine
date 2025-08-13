use crate::editor::Editor;
use macroquad::prelude::*;
mod editor;
mod tilemap;
mod gui;
mod world;
mod room;

#[macroquad::main("Map Editor")]
async fn main() {
    let mut editor = Editor::new(12, 9);

    loop {
        editor.update();
        editor.draw();
        next_frame().await
    }
}