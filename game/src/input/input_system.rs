// game/src/input/player_input.rs
use engine_core::input::input_snapshot::*;
use engine_core::game::game::Game;
use engine_core::ecs::component::Velocity;

/// Walk speed in world units per second.
const PLAYER_SPEED: f32 = 100.0;
/// Jump speed in world units per second.
pub const JUMP_SPEED: f32 = 250.0; 

pub fn update_player_input(game: &mut Game) {
    let world = game.current_world_mut();

    let player = world.world_ecs.get_player_entity();

    let velocity = world.world_ecs
        .get_store_mut::<Velocity>()
        .get_mut(player)
        .expect("Player must have a Velocity component");

    // Update velocity
    let horizontal_dir = get_horizontal_input();
    velocity.x = horizontal_dir * PLAYER_SPEED;

    if jump() && velocity.y == 0.0 {
        velocity.y = -JUMP_SPEED;
    }
}