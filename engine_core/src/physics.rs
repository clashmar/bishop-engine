use crate::{constants::*, player::PlayerOld, tiles::tilemap::TileMap};

pub fn update_physics(entity: &mut PlayerOld, map: &TileMap) {
        // apply_gravity(entity);
        // resolve_horizontal_movement(entity, map);
        // resolve_vertical_movement(entity, map);
        // clamp_position(entity, map);
    }

fn apply_gravity(entity: &mut PlayerOld) { 
    entity.velocity_y += GRAVITY;
}

fn clamp_position(entity: &mut PlayerOld, map: &TileMap) { 
    let max_x = (map.width as f32 * TILE_SIZE) - PLAYER_WIDTH;
    entity.actual_position.x = entity.actual_position.x.clamp(0.0, max_x);
}

fn resolve_vertical_movement(entity: &mut PlayerOld, map: &TileMap) {
    let map_pixel_height = map.height as f32 * TILE_SIZE;

    // Predict next vertical position
    let next_actual_y = (entity.actual_position.y + entity.velocity_y)
        .clamp(0.0, map_pixel_height - PLAYER_HEIGHT);

    // Convert to cartesian bottom y (bottom-up)
    let mut cartesian_bottom_y = map_pixel_height - next_actual_y - PLAYER_HEIGHT;

    // Collision with floor when falling
    if entity.velocity_y > 0.0 {
        let grid_y = TileMap::pixel_to_grid(cartesian_bottom_y);
        if grid_y >= 0 {
            let left_tile_x = TileMap::pixel_to_grid(entity.actual_position.x);
            let right_tile_x = TileMap::pixel_to_grid(entity.actual_position.x + PLAYER_WIDTH - 1.0);

            // let collided = TileMap::any_tiles_in_range(
            //     map,
            //     left_tile_x..=right_tile_x,
            //     grid_y..=grid_y,
            //     |tile| tile.is_walkable,
            // );

            // if collided {
            //     // Find the tile top y to snap to floor
            //     let tile_top_y = (grid_y as f32 + 1.0) * TILE_SIZE;
            //     let prev_cartesian_bottom_y = map_pixel_height - entity.actual_position.y - PLAYER_HEIGHT;

            //     if cartesian_bottom_y < tile_top_y && prev_cartesian_bottom_y >= tile_top_y {
            //         // Snap to floor
            //         cartesian_bottom_y = tile_top_y;
            //         entity.velocity_y = 0.0;
            //         entity.is_airborne = false;
            //         entity.has_double_jump = true;
            //     }
            // }
        }
    }

    // Collision with ceiling when moving up
    if entity.velocity_y < 0.0 {
        let next_top_y = next_actual_y; // predicted top after move
        let cartesian_top_y = map_pixel_height - next_top_y;
        let grid_y_top = TileMap::pixel_to_grid(cartesian_top_y);
        let left_tile_x = TileMap::pixel_to_grid(entity.actual_position.x);
        let right_tile_x = TileMap::pixel_to_grid(entity.actual_position.x + PLAYER_WIDTH - 1.0);

        // let collided = TileMap::any_tiles_in_range(
        //     map,
        //     left_tile_x..=right_tile_x,
        //     grid_y_top..=grid_y_top,
        //     |tile| tile.is_solid,
        // );

        // if collided {
        //     // Compute tile bottom y
        //     let tile_bottom_y = map_pixel_height - (grid_y_top as f32) * TILE_SIZE;

        //     if next_top_y <= tile_bottom_y {
        //         // Clamp next_actual_y to tile bottom
        //         cartesian_bottom_y = map_pixel_height - tile_bottom_y - PLAYER_HEIGHT;
        //         entity.velocity_y = 0.0;
        //         entity.is_airborne = true;
        //     }
        // }
    }

    // Apply vertical position update after collision resolution
    entity.actual_position.y = map_pixel_height - cartesian_bottom_y - PLAYER_HEIGHT;
}

fn resolve_horizontal_movement(entity: &mut PlayerOld, map: &TileMap) {
    let map_pixel_height = map.height as f32 * TILE_SIZE;

    let next_x = entity.actual_position.x + entity.velocity_x;
    let player_top = entity.actual_position.y;
    let player_bottom = entity.actual_position.y + PLAYER_HEIGHT;

    // Convert vertical pixel positions to cartesian y (bottom-up)
    let cartesian_player_top = map_pixel_height - player_top;
    let cartesian_player_bottom = map_pixel_height - player_bottom;

    // To avoid overlap issues, subtract a small epsilon
    let epsilon = 0.01;

    // Use floor for bottom, with a tiny offset inside the tile
    let tile_bottom_y = TileMap::pixel_to_grid(cartesian_player_bottom + epsilon);

    // Use floor for top, subtract epsilon to avoid including tile above if not overlapping
    let tile_top_y = TileMap::pixel_to_grid(cartesian_player_top - epsilon);

    let tile_left_x = TileMap::pixel_to_grid(next_x);
    let tile_right_x = TileMap::pixel_to_grid(next_x + PLAYER_WIDTH - 1.0);

    // let blocked;

    if entity.velocity_x > 0.0 {
        let check_x = tile_right_x;

        // blocked = TileMap::any_tiles_in_range(
        //     map,
        //     check_x..=check_x,
        //     tile_bottom_y..=tile_top_y,
        //     |tile| tile.is_solid,
        // );

        // if blocked {
        //     entity.actual_position.x = (check_x as f32 * TILE_SIZE) - PLAYER_WIDTH;
        //     entity.velocity_x = 0.0;
        // } else {
        //     entity.actual_position.x = next_x;
        // }
    } else if entity.velocity_x < 0.0 {
        let check_x = tile_left_x;

        // blocked = TileMap::any_tiles_in_range(
        //     map,
        //     check_x..=check_x,
        //     tile_bottom_y..=tile_top_y,
        //     |tile| tile.is_solid,
        // );

        // if blocked {
        //     entity.actual_position.x = (check_x as f32 + 1.0) * TILE_SIZE;
        //     entity.velocity_x = 0.0;
        // } else {
        //     entity.actual_position.x = next_x;
        // }
    }

    // Clamp horizontal position to map bounds
    let max_x = (map.width as f32 * TILE_SIZE) - PLAYER_WIDTH;
    entity.actual_position.x = entity.actual_position.x.clamp(0.0, max_x);
}