// game/src/physics/physics_system.rs
use macroquad::prelude::Vec2;
use engine_core::{
    ecs::{
        component::{Collider, PhysicsBody, Position, Velocity}, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, 
    world::room::Room
};
use uuid::Uuid;
use crate::{
    constants::*, 
    physics::collision::sweep_move, 
    world::world_helpers::*
};

/// Fixed time‑step (seconds).
const DT: f32 = 1.0 / 60.0;

/// Applies physics to all entities with a `PhysicsBody` component.
/// Returns `Some((entity, exit_id, position))` when an entity crosses an exit, otherwise `None`.
pub fn update_physics(
    world_ecs: &mut WorldEcs,
    room: &Room,
) -> Option<(Entity, Uuid, Vec2)> {
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

        vel_cur.y += GRAVITY * DT;
        let delta = Vec2::new(vel_cur.x * DT, vel_cur.y * DT);

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
        if let Some(target_id) = crossed_exit(new_pos, sweep.allowed_delta, &collider, room) {
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