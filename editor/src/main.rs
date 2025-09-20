// editor/src/main.rs
use crate::editor::Editor;
use engine_core::constants::{WORLD_VIRTUAL_HEIGHT, WORLD_VIRTUAL_WIDTH};
use macroquad::prelude::*;

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

fn window_conf() -> Conf {
    Conf {
        window_title: "Bishop Engine".to_owned(),
        window_height: WORLD_VIRTUAL_HEIGHT as i32,
        window_width: WORLD_VIRTUAL_WIDTH as i32,
        fullscreen: true,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    let mut editor = Editor::new().await?;

    loop {
        editor.update().await;
        editor.draw();
        next_frame().await
    }
}

