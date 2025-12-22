// game/src/physics/collision.rs
use engine_core::tiles::tile::TileComponent;
use engine_core::engine_global::tile_size;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::tiles::tilemap::TileMap;
use engine_core::ecs::component::*;
use engine_core::world::room::Exit;
use std::collections::HashSet;
use macroquad::prelude::*;

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
    exits: &[Exit],
) -> SweepResult {
    // Gather every solid AABB to test against
    let mut obstacles: Vec<(Vec2, Vec2)> = Vec::new();

    // Tiles
    // Only tiles that carry a Solid component are obstacles
    for ((x, y), tile_def_id) in tilemap.tiles.iter() {
        let Some(tile_def) = world_ecs.tile_defs.get(tile_def_id) else {continue};

        if tile_def.components.contains(&TileComponent::Solid(true)) {
            let tile_pos = room_origin + vec2(*x as f32 * tile_size(), *y as f32 * tile_size());
            let tile_aabb = (tile_pos, tile_pos + vec2(tile_size(), tile_size()));
            obstacles.push(tile_aabb);
        }
    }

    // Create an invisible border around the edge of the room except where exits are placed
    add_border_obstacles(&mut obstacles, room_origin, tilemap, exits);

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

fn add_border_obstacles(
    obstacles: &mut Vec<(Vec2, Vec2)>,
    room_origin: Vec2,
    tilemap: &TileMap,
    exits: &[Exit],
) {
    let ts = tile_size();
    let w = tilemap.width as i32;
    let h = tilemap.height as i32;

    let mut outer_exits: HashSet<(i32, i32)> = HashSet::with_capacity(exits.len());
    for e in exits {
        outer_exits.insert((e.position.x as i32, e.position.y as i32));
    }

    for gx in 0..w {
        if !outer_exits.contains(&(gx, -1)) {
            let min = room_origin + vec2(gx as f32 * ts, -ts);
            obstacles.push((min, min + vec2(ts, ts)));
        }
    }

    for gx in 0..w {
        if !outer_exits.contains(&(gx, h)) {
            let min = room_origin + vec2(gx as f32 * ts, h as f32 * ts);
            obstacles.push((min, min + vec2(ts, ts)));
        }
    }

    for gy in 0..h {
        if !outer_exits.contains(&(-1, gy)) {
            let min = room_origin + vec2(-ts, gy as f32 * ts);
            obstacles.push((min, min + vec2(ts, ts)));
        }
    }

    for gy in 0..h {
        if !outer_exits.contains(&(w, gy)) {
            let min = room_origin + vec2(w as f32 * ts, gy as f32 * ts);
            obstacles.push((min, min + vec2(ts, ts)));
        }
    }
}