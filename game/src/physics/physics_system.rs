// game/src/physics/physics_system.rs
use macroquad::prelude::Vec2;
use engine_core::{
    ecs::{
        component::{Collider, PhysicsBody, Position, Velocity}, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, world::room::Room
};
use crate::{
    constants::*, 
    physics::collision::sweep_move, 
    world::world_helpers::*
};

/// Applies physics to all entities with a `PhysicsBody` component.
/// Returns `Some((entity, exit_id, position))` when an entity crosses an exit, otherwise `None`.
pub fn update_physics(
    world_ecs: &mut WorldEcs,
    room: &Room,
    dt: f32,
) -> Option<(Entity, usize, Vec2)> {
    let tilemap = &room.variants[0].tilemap;
    let entities: Vec<_> = world_ecs
        .get_store::<PhysicsBody>()
        .data
        .keys()
        .cloned()
        .collect();

    for entity in entities {
        let (pos_cur, mut vel_cur, collider) = {
            let p = world_ecs.get::<Position>(entity).unwrap();
            let v = world_ecs.get::<Velocity>(entity).unwrap();
            let c = world_ecs
                .get::<Collider>(entity)
                .cloned()
                .unwrap_or_default();
            (p.position, *v, c)
        };

        vel_cur.y += GRAVITY * dt;

        let delta = Vec2::new(vel_cur.x * dt, vel_cur.y * dt);

        let sweep = sweep_move(
            world_ecs,
            tilemap,
            room.position,
            pos_cur,
            delta,
            collider,
        );

        let new_pos = pos_cur + sweep.allowed_delta;
        let mut new_vel = vel_cur;

        if sweep.blocked_x {
            new_vel.x = 0.0;
        }
        if sweep.blocked_y {
            new_vel.y = 0.0;
        }

        // Exit check
        if let Some(target_id) = crossed_exit(new_pos, delta, &collider, room) {
            {
                let pos_mut = world_ecs.get_mut::<Position>(entity).unwrap();
                pos_mut.position = new_pos;
            }
            {
                let vel_mut = world_ecs.get_mut::<Velocity>(entity).unwrap();
                *vel_mut = new_vel;
            }
            return Some((entity, target_id, new_pos));
        }

        // Clamp to room
        let clamped = clamp_to_room(new_pos, &collider, room);

        {
            let pos_mut = world_ecs.get_mut::<Position>(entity).unwrap();
            pos_mut.position = clamped;
        }
        {
            let vel_mut = world_ecs.get_mut::<Velocity>(entity).unwrap();
            *vel_mut = new_vel;
        }
    }
    None
}