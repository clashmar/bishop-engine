use crate::input;
use macroquad::{color::Color, math::Vec2, prelude::*};
use::core::{tile::TileType, map::TileMap, constants::*};

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub grid_position: IVec2,
    pub actual_position: Vec2,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub is_airborne: bool,
    pub has_double_jump: bool,   
    pub color: Color,
}

impl Entity {
    pub fn update_physics(&mut self, map: &TileMap, gravity: f32, map_height: usize) {
        let player_height = TILE_SIZE;
        let player_width = TILE_SIZE;
        let map_pixel_height = map_height as f32 * TILE_SIZE;

        // --- Horizontal movement with collision ---
        // Predict horizontal position after applying velocity
        let next_x = self.actual_position.x + self.velocity_x;

        // Calculate vertical bounds of player
        let player_top = self.actual_position.y;
        let player_bottom = self.actual_position.y + player_height;

        // Compute vertical tile range player occupies
        // We convert screen y to cartesian y (bottom-up) for tile indexing:
        let cartesian_player_top = map_pixel_height - player_top;
        let cartesian_player_bottom = map_pixel_height - player_bottom;

        // Tile Y indices the player spans (top to bottom)
        let tile_top_y = ((cartesian_player_top) / TILE_SIZE).floor() as i32;
        let tile_bottom_y = ((cartesian_player_bottom) / TILE_SIZE).floor() as i32;

        // Determine horizontal tile indices for next_x position
        let tile_left_x = (next_x / TILE_SIZE).floor() as i32;
        let tile_right_x = ((next_x + player_width - 1.0) / TILE_SIZE).floor() as i32;

        let mut blocked = false;

        if self.velocity_x > 0.0 {
            // Moving right - check the tiles to the right edge
            let check_x = tile_right_x;

            for tile_y in tile_bottom_y..=tile_top_y {
                if tile_y >= 0 && tile_y < map_height as i32 && check_x >= 0 && check_x < map.width as i32 {
                    if let Some(tile) = map.get_tile(check_x as usize, tile_y as usize) {
                        if tile.is_solid {
                            // The player vertically overlaps with this tile, so block movement
                            blocked = true;
                            break;
                        }
                    }
                }
            }

            if blocked {
                // Snap player to just before the tile boundary on the right
                self.actual_position.x = (check_x as f32 * TILE_SIZE) - player_width;
                self.velocity_x = 0.0;
            } else {
                self.actual_position.x = next_x;
            }
        } else if self.velocity_x < 0.0 {
            // Moving left - check tiles to the left edge
            let check_x = tile_left_x;

            for tile_y in tile_bottom_y..=tile_top_y {
                if tile_y >= 0 && tile_y < map_height as i32 && check_x >= 0 && check_x < map.width as i32 {
                    if let Some(tile) = map.get_tile(check_x as usize, tile_y as usize) {
                        if tile.is_solid {
                            blocked = true;
                            break;
                        }
                    }
                }
            }

            if blocked {
                // Snap player to just after the tile boundary on the left
                self.actual_position.x = (check_x as f32 + 1.0) * TILE_SIZE;
                self.velocity_x = 0.0;
            } else {
                self.actual_position.x = next_x;
            }
        } else {
            // No horizontal velocity, just keep current position
        }

        // Clamp horizontal position to map bounds
        let max_x = (map.width as f32 * TILE_SIZE) - player_width;
        self.actual_position.x = self.actual_position.x.clamp(0.0, max_x);

        // --- Vertical movement ---

        // Apply gravity to vertical velocity
        self.velocity_y += gravity;

        // Calculate next vertical position based on velocity
        let next_actual_y = (self.actual_position.y + self.velocity_y).clamp(0.0, map_pixel_height - player_height);

        // Convert to cartesian bottom y (bottom-up)
        let mut cartesian_bottom_y = map_pixel_height - next_actual_y - player_height;

        let grid_x = (self.actual_position.x / TILE_SIZE) as usize;
        let grid_y = (cartesian_bottom_y / TILE_SIZE).floor() as i32;

        // Check collision with floor (only if falling)
        if self.velocity_y > 0.0 {
            if grid_y >= 0 {
                // Player's left and right edge in tile coordinates
                let left_tile_x = (self.actual_position.x / TILE_SIZE).floor() as i32;
                let right_tile_x = ((self.actual_position.x + TILE_SIZE - 1.0) / TILE_SIZE).floor() as i32;

                for tile_x in left_tile_x..=right_tile_x {
                    if tile_x >= 0 && tile_x < map.width as i32 {
                        if let Some(tile) = map.get_tile(tile_x as usize, grid_y as usize) {
                            if tile.is_walkable {
                                let tile_top_y = (grid_y as f32 + 1.0) * TILE_SIZE;
                                let prev_cartesian_bottom_y = map_pixel_height - self.actual_position.y - player_height;

                                if cartesian_bottom_y < tile_top_y && prev_cartesian_bottom_y >= tile_top_y {
                                    cartesian_bottom_y = tile_top_y;
                                    self.velocity_y = 0.0;
                                    self.is_airborne = false;
                                    self.has_double_jump = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Convert back to screen coordinates
        self.actual_position.y = map_pixel_height - cartesian_bottom_y - player_height;
    }


    pub fn handle_horizontal_input(&mut self) {
        let input = input::get_horizontal_input();

        let acceleration = if self.is_airborne { 0.5 } else { 1.0 };
        let max_speed = 6.0;
        
        self.velocity_x += input * acceleration;
        self.velocity_x = self.velocity_x.clamp(-max_speed, max_speed);

        // Friction
        if input == 0.0 {
            let friction = if self.is_airborne { 0.05 } else { 0.3 };
            self.velocity_x *= 1.0 - friction;

            if self.velocity_x.abs() < 0.1 {
                self.velocity_x = 0.0;
            }
        }
    }

    pub fn update_grid_position(&mut self, map_height: usize) -> bool {
        let new_grid_x = (self.actual_position.x / TILE_SIZE).floor() as i32;
        let screen_y = self.actual_position.y;
        let new_grid_y = (map_height as f32 - 1.0 - (screen_y / TILE_SIZE)).floor() as i32;

        let new_grid = ivec2(new_grid_x, new_grid_y);

        if new_grid != self.grid_position {
            self.grid_position = new_grid;
            println!("Player moved to grid position: {:?}", new_grid);
            true
        } else {
            false
        }
    }
}