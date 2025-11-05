use game_lib::game::GameState;

#[macroquad::main("Demo")]
async fn main() {
    let mut game = GameState::new().await;
    game.run_game_loop().await;
}
