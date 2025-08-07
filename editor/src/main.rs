use crate::editor::EditorState;
use macroquad::prelude::*;
mod editor;

#[macroquad::main("Map Editor")]
async fn main() {
    let mut editor = EditorState::new(12, 9);

    loop {
        editor.update();
        editor.draw();
        next_frame().await
    }
}