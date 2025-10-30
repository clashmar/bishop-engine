use engine_core::{
    ecs::component::Collider, 
    global::tile_size, 
    world::room::{ExitDirection, Room}
};
use macroquad::prelude::*;
use uuid::Uuid;

pub fn crossed_exit(
    entity_position: Vec2,
    delta: Vec2,               
    collider: &Collider,
    room: &Room,
) -> Option<Uuid> {
    // Position after the movement
    let new_pos = entity_position + delta;
    let entity_rect = Rect::new(new_pos.x, new_pos.y, collider.width, collider.height);

    let room_min = room.position;
    let room_max = room.position + room.size * tile_size();

    for exit in &room.exits {
        // World‑space rectangle of the exit tile
        let exit_world = room.position + exit.position * tile_size();
        let exit_rect = Rect::new(exit_world.x, exit_world.y, tile_size(), tile_size());

        // No overlap
        if !entity_rect.overlaps(&exit_rect) {
            continue;
        }

        // Size fit check
        if !fits_exit(entity_rect, exit_rect, exit.direction) {
            continue;
        }

        // The entity overlaps the exit
        let crossed = match exit.direction {
            ExitDirection::Up => delta.y < 0.0 && entity_rect.top() <= room_min.y,
            ExitDirection::Down => delta.y > 0.0 && entity_rect.bottom() >= room_max.y,
            ExitDirection::Left => delta.x < 0.0 && entity_rect.left() <= room_min.x,
            ExitDirection::Right => delta.x > 0.0 && entity_rect.right() >= room_max.x,
        };

        if crossed {
            return exit.target_room_id;
        }
    }
    None
}

/// Clamps a collider within the bounds of a room and returns the new position.
pub fn clamp_to_room(mut pos: Vec2, collider: &Collider, room: &Room) -> Vec2 {
    let (room_min, room_max) = room.room_bounds();
    let max_x = (room_max.x - collider.width).max(room_min.x);
    let max_y = (room_max.y - collider.height).max(room_min.y);
    pos.x = pos.x.clamp(room_min.x, max_x);
    pos.y = pos.y.clamp(room_min.y, max_y);
    pos
}

/// Returns `true` when the entity’s size can actually pass through the
/// given exit tile in the specified direction.
fn fits_exit(entity_rect: Rect, exit_rect: Rect, direction: ExitDirection) -> bool {
    // Percent of an entity that needs to be within the 'light' of an exit
    const TOLERANCE: f32 = 0.90;
     
    let overlap_length = match direction {
        ExitDirection::Left | ExitDirection::Right => {
            // Vertical overlap
            (entity_rect.bottom().min(exit_rect.bottom())
                - entity_rect.top().max(exit_rect.top()))
                .max(0.0) // Clamp negative values
        }
        ExitDirection::Up | ExitDirection::Down => {
            // Horizontal overlap
            (entity_rect.right().min(exit_rect.right())
                - entity_rect.left().max(exit_rect.left()))
                .max(0.0) // Clamp negative values
        }
    };

    // Required minimum overlap
    let required = match direction {
        ExitDirection::Left | ExitDirection::Right => {
            TOLERANCE * entity_rect.h.min(tile_size())
        }
        ExitDirection::Up | ExitDirection::Down => {
            TOLERANCE * entity_rect.w.min(tile_size())
        }
    };

    overlap_length >= required
}


