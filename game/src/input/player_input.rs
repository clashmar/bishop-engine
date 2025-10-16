// game/src/input/player_input.rs
use engine_core::{ecs::component::Velocity, input::get_omni_input};

/// Walk‑speed in world units per second.
const PLAYER_SPEED: f32 = 100.0;

pub fn update_player_input(velocity: &mut Velocity) {
    // Update velocity
    let input_dir = get_omni_input();
    velocity.x = input_dir.x * PLAYER_SPEED;
    velocity.y = input_dir.y * PLAYER_SPEED;
}