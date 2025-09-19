use game_lib::game::GameState;
use macroquad::prelude::*;

#[macroquad::main("Demo")]
async fn main() {
    let mut game = GameState::new().await;

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
