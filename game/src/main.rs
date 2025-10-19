use engine_core::constants::{FIXED_DT, MAX_ACCUM};
use game_lib::game::GameState;
use macroquad::prelude::*;

#[macroquad::main("Demo")]
async fn main() {
    let mut game = GameState::new().await;
    let mut accumulator = 0.0_f32;

    loop {
        // Time elapsed since last frame
        let frame_dt = get_frame_time();
        accumulator += frame_dt;

        // Clamp the backlog
        if accumulator > MAX_ACCUM {
            accumulator = MAX_ACCUM;
        }

        // Fixed‑step physics
        while accumulator >= FIXED_DT {
            game.fixed_update(FIXED_DT);
            accumulator -= FIXED_DT;
        }

        // Per frame async work (animation, streaming, etc.)
        game.update_async(frame_dt).await;

        // Interpolation factor for rendering
        let alpha = accumulator / FIXED_DT;
        game.render(alpha);
        next_frame().await;
    }
}
