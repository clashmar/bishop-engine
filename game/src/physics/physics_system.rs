// game/src/physics/physics_system.rs
use crate::physics::collision::sweep_move;
use crate::constants::GRAVITY; 
use engine_core::prelude::*;
use bishop::prelude::*;

/// Applies physics to all entities with a `PhysicsBody` component.
pub fn update_physics(
    asset_manager: &AssetManager,
    ecs: &mut Ecs,
    room: &Room,
    dt: f32,
    grid_size: f32,
) {
    let tilemap = &room.variants[room.current_variant_index()].tilemap;
    
    let entities: Vec<_> = ecs
        .get_store::<PhysicsBody>()
        .data
        .keys()
        .cloned()
        .collect();

    for entity in entities {
        let (pos_cur, pivot, mut vel_cur, collider) = {
            let t = ecs.get::<Transform>(entity).unwrap();
            let v = ecs.get::<Velocity>(entity).unwrap();
            let c = ecs
                .get::<Collider>(entity)
                .cloned()
                .unwrap_or_default();
            (t.position, t.pivot, *v, c)
        };

        vel_cur.y += GRAVITY * dt;

        let delta = Vec2::new(vel_cur.x * dt, vel_cur.y * dt);

        let sweep = sweep_move(
            asset_manager,
            ecs,
            tilemap,
            room.position,
            pos_cur,
            delta,
            collider,
            pivot,
            &room.exits,
            grid_size,
        );

        let new_pos = pos_cur + sweep.allowed_delta;
        let mut new_vel = vel_cur;

        if sweep.blocked_x {
            new_vel.x = 0.0;
        }
        if sweep.blocked_y {
            new_vel.y = 0.0;
        }

        update_entity_position(ecs, entity, new_pos);

        {
            let vel_mut = ecs.get_mut::<Velocity>(entity).unwrap();
            *vel_mut = new_vel;
        }

        // Grounded when blocked_y while moving down
        if let Some(grounded) = ecs.get_mut::<Grounded>(entity) {
            grounded.0 = sweep.blocked_y && vel_cur.y >= 0.0;
        }
    }
}