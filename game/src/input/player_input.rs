// game/src/input/player_input.rs
use engine_core::{ecs::component::Velocity, input::*};

/// Walk speed in world units per second.
const PLAYER_SPEED: f32 = 125.0;
/// Jump speed in world units per second.
pub const JUMP_SPEED: f32 = 250.0; 

pub fn update_player_input(velocity: &mut Velocity) {
    // Update velocity
    let horizontal_dir = get_horizontal_input();
    velocity.x = horizontal_dir * PLAYER_SPEED;

    if jump() && velocity.y == 0.0 {
        velocity.y = -JUMP_SPEED;
    }
}