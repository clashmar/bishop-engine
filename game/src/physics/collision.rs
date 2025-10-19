// game/src/physics/collision.rs
use macroquad::prelude::*;
use engine_core::{
    ecs::{
        component::{Collider, Position, Solid},
        world_ecs::WorldEcs,
    }, 
    global::tile_size, 
    tiles::tilemap::TileMap
};

const OVERLAP_EPS: f32 = 0.0001; 

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
    if delta == 0.0 {
        return (0.0, false);
    }

    let mut allowed = delta;
    let mut blocked = false;

    let (my_min, my_max) = if axis == 0 {
        (position.x, position.x + this_size.x)
    } else {
        (position.y, position.y + this_size.y)
    };

    for (obs_min, obs_max) in obstacles.iter() {
        let (obs_min_axis, obs_max_axis) = if axis == 0 {
            (obs_min.x, obs_max.x)
        } else {
            (obs_min.y, obs_max.y)
        };

        // Overlap on the other axis
        let overlap_other = if axis == 0 {
            !(position.y + this_size.y <= obs_min.y + OVERLAP_EPS
                || position.y >= obs_max.y - OVERLAP_EPS)
        } else {
            !(position.x + this_size.x <= obs_min.x + OVERLAP_EPS
                || position.x >= obs_max.x - OVERLAP_EPS)
        };

        if !overlap_other {
            continue;
        }

        // Apply directional epsilon for movement axis
        if delta > 0.0 {
            // Moving positive (right or down)
            if my_max <= obs_min_axis + OVERLAP_EPS && my_max + delta > obs_min_axis {
                let dist = obs_min_axis - my_max;
                if dist < allowed {
                    allowed = dist;
                    blocked = true;
                }
            }
        } else {
            // Moving negative (left or up)
            if my_min >= obs_max_axis - OVERLAP_EPS && my_min + delta < obs_max_axis {
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
                let tile_pos = room_origin + vec2(*x as f32 * tile_size(), *y as f32 * tile_size());
                let tile_aabb = (tile_pos, tile_pos + vec2(tile_size(), tile_size()));
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

    obstacles.extend(room_bounds_aabbs(room_origin, tilemap.width, tilemap.height));

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

/// Returns four AABBs that represent the four borders of a rectangular room.
fn room_bounds_aabbs(origin: Vec2, map_width: usize, map_height: usize) -> Vec<(Vec2, Vec2)> {
    let width = map_width as f32 * tile_size();
    let height = map_height as f32 * tile_size();

    let thickness = 0.1_f32;

    // Left wall
    let left = (origin - vec2(thickness, 0.0), origin + vec2(0.0, height));

    // Right wall
    let right = (origin + vec2(width, 0.0), origin + vec2(width + thickness, height));

    // Top wall
    let top = (origin - vec2(0.0, thickness), origin + vec2(width, 0.0));

    // Bottom wall
    let bottom = (origin + vec2(0.0, height), origin + vec2(width, height + thickness));

    vec![left, right, top, bottom]
}