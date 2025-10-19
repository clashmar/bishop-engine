use engine_core::constants::{FIXED_DT, MAX_ACCUM};
use game_lib::game::GameState;
use macroquad::prelude::*;

#[macroquad::main("Demo")]
async fn main() {
    let mut game = GameState::new().await;
    let mut accumulator = 0.0_f32;

    loop {
        let frame_dt = get_frame_time();
        accumulator += frame_dt;
        if accumulator > MAX_ACCUM {
            accumulator = MAX_ACCUM;
        }

        while accumulator >= FIXED_DT {
            game.fixed_update(FIXED_DT).await;
            accumulator -= FIXED_DT;
        }

        let alpha = accumulator / FIXED_DT;
        game.render(alpha);
        next_frame().await;
    }
}
