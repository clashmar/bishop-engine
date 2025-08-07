mod entity;
mod camera;
mod game;
mod input;
mod modes;

use crate::game::GameState;
use macroquad::prelude::*;

#[macroquad::main("Tilemap Demo")]
async fn main() {
    let mut game = GameState::new();

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
