// game/src/physics/collision.rs
use macroquad::prelude::*;
use engine_core::{
    ecs::{
        component::{Collider, Position, Solid},
        world_ecs::WorldEcs,
    },
    tiles::tilemap::TileMap,
};
use engine_core::constants::*;

/// Information returned by the sweep test.
pub struct SweepResult {
    /// The displacement that can actually be applied without intersecting anything.
    pub allowed_delta: Vec2,
    /// Was the X‑axis blocked?
    pub blocked_x: bool,
    /// Was the Y‑axis blocked?
    pub blocked_y: bool,
}

/// Build an axis‑aligned bounding box (AABB) from a position + collider.
#[inline]
fn aabb(position: Vec2, collider: Collider) -> (Vec2, Vec2) {
    // (min, max)
    (position, position + Vec2::new(collider.width, collider.height))
}

/// Resolve a single axis (X or Y).
fn resolve_axis(
    position: Vec2,
    delta: f32,
    axis: usize,
    this_size: Vec2,
    obstacles: &[(Vec2, Vec2)],
) -> (f32, bool) {
    // No movement 
    if delta == 0.0 {
        return (0.0, false);
    }

    // Desired new coordinate on this axis
    let mut allowed = delta;
    let mut blocked = false;

    // Current min/max on the moving axis
    let (my_min, my_max) = if axis == 0 {
        (position.x, position.x + this_size.x)
    } else {
        (position.y, position.y + this_size.y)
    };

    // Scan every obstacle and shrink the allowed movement if this would hit it
    for (obs_min, obs_max) in obstacles.iter() {
        // Get the obstacle’s interval on the same axis
        let (obs_min_axis, obs_max_axis) = if axis == 0 {
            (obs_min.x, obs_max.x)
        } else {
            (obs_min.y, obs_max.y)
        };

        // Only care about obstacles that overlap on the other axis
        let overlap_other = if axis == 0 {
            // Y‑intervals must intersect
            !(position.y + this_size.y <= obs_min.y || position.y >= obs_max.y)
        } else {
            // X‑intervals must intersect
            !(position.x + this_size.x <= obs_min.x || position.x >= obs_max.x)
        };

        if !overlap_other {
            continue;
        }

        if delta > 0.0 {
            // Moving positive direction – this will hit the obstacle’s left side
            if my_max <= obs_min_axis && my_max + delta > obs_min_axis {
                let dist = obs_min_axis - my_max;
                if dist < allowed {
                    allowed = dist;
                    blocked = true;
                }
            }
        } else {
            // Moving negative direction – we will hit the obstacle’s right side
            if my_min >= obs_max_axis && my_min + delta < obs_max_axis {
                let dist = obs_max_axis - my_min;
                if dist > allowed {
                    allowed = dist;
                    blocked = true;
                }
            }
        }
    }
    (allowed, blocked)
}

/// Sweep the requested movement and return the maximal safe delta.
pub fn sweep_move(
    world_ecs: &mut WorldEcs,
    tilemap: &TileMap,
    room_origin: Vec2,               
    entity_position: Vec2,                 
    desired_delta: Vec2,
    collider: Collider,
) -> SweepResult {
    // Gather every solid AABB we have to test against
    let mut obstacles: Vec<(Vec2, Vec2)> = Vec::new();

    // Tiles
    // Only tiles that carry a Solid component are obstacles
    for ((x, y), tile) in tilemap.tiles.iter() {
        let Some(entity) = tile.entity else { continue };
        if let Some(solid) = world_ecs.get::<Solid>(entity) {
            if solid.0 {
                let tile_pos = room_origin + vec2(*x as f32 * TILE_SIZE, *y as f32 * TILE_SIZE);
                let tile_aabb = (tile_pos, tile_pos + vec2(TILE_SIZE, TILE_SIZE));
                obstacles.push(tile_aabb);
            }
        }
    }

    // Other solid entities
    // Iterate over every Collider component in the world, skip the moving one
    for (other_entity, other_coll) in world_ecs.get_store::<Collider>().data.iter() {
        // Do not test against ourselves
        if let Some(other_pos) =
            world_ecs.get::<Position>(*other_entity)
        {
            if (other_pos.position - entity_position).length() < 0.001 {
                // Same entity
                continue;
            }
        }

        // Only solid entities block movement
        if let Some(solid) = world_ecs.get::<Solid>(*other_entity) {
            if solid.0 {
                if let Some(other_pos) =
                    world_ecs.get::<Position>(*other_entity)
                {
                    let aabb = aabb(other_pos.position, *other_coll);
                    obstacles.push(aabb);
                }
            }
        }
    }

    // Sweep X axis, then Y axis
    let (allowed_x, blocked_x) = resolve_axis(
        entity_position,
        desired_delta.x,
        0,
        Vec2::new(collider.width, collider.height),
        &obstacles,
    );

    // Apply the X movement before testing Y
    let pos_after_x = entity_position + Vec2::new(allowed_x, 0.0);
    let (allowed_y, blocked_y) = resolve_axis(
        pos_after_x,
        desired_delta.y,
        1,
        Vec2::new(collider.width, collider.height),
        &obstacles,
    );

    SweepResult {
        allowed_delta: Vec2::new(allowed_x, allowed_y),
        blocked_x,
        blocked_y,
    }
}