use crate::input;
use macroquad::{color::Color, math::Vec2, prelude::*};
use::core::{map::TileMap, constants::*};

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
        let player_width = PLAYER_WIDTH;
        let player_height = PLAYER_HEIGHT;
        let map_pixel_height = map_height as f32 * TILE_SIZE;

        // --- Vertical movement ---

        // Apply gravity
        self.velocity_y += gravity;

        // Predict next vertical position (top-left coordinate)
        let next_actual_y = (self.actual_position.y + self.velocity_y).clamp(0.0, map_pixel_height - player_height);

        // Convert to cartesian bottom y (bottom-up)
        let mut cartesian_bottom_y = map_pixel_height - next_actual_y - player_height;

        // Tile coords for bottom of player
        let grid_x = (self.actual_position.x / TILE_SIZE) as usize;
        let grid_y = (cartesian_bottom_y / TILE_SIZE).floor() as i32;

        // --- Collision with floor when falling ---
        if self.velocity_y > 0.0 {
            if grid_y >= 0 {
                let left_tile_x = (self.actual_position.x / TILE_SIZE).floor() as i32;
                let right_tile_x = ((self.actual_position.x + player_width - 1.0) / TILE_SIZE).floor() as i32;

                for tile_x in left_tile_x..=right_tile_x {
                    if tile_x >= 0 && tile_x < map.width as i32 {
                        if let Some(tile) = map.get_tile(tile_x as usize, grid_y as usize) {
                            if tile.is_walkable {
                                let tile_top_y = (grid_y as f32 + 1.0) * TILE_SIZE;
                                let prev_cartesian_bottom_y = map_pixel_height - self.actual_position.y - player_height;

                                if cartesian_bottom_y < tile_top_y && prev_cartesian_bottom_y >= tile_top_y {
                                    // Snap to floor
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

        // --- Collision with ceiling when jumping (moving up) ---
        if self.velocity_y < 0.0 {
            let mut clamped = false;
            let next_top_y = next_actual_y; // predicted top after move
            let cartesian_top_y = map_pixel_height - next_top_y;
            let grid_y_top = (cartesian_top_y / TILE_SIZE).floor() as i32;
            let left_tile_x = (self.actual_position.x / TILE_SIZE).floor() as i32;
            let right_tile_x = ((self.actual_position.x + player_width - 1.0) / TILE_SIZE).floor() as i32;

            for tile_x in left_tile_x..=right_tile_x {
                if tile_x >= 0 && tile_x < map.width as i32 && grid_y_top >= 0 && grid_y_top < map_height as i32 {
                    if let Some(tile) = map.get_tile(tile_x as usize, grid_y_top as usize) {
                        if tile.is_solid {
                            let tile_bottom_y = map_pixel_height - (grid_y_top as f32) * TILE_SIZE;

                            if next_top_y <= tile_bottom_y {
                                // Clamp next_actual_y to tile bottom
                                let clamped_next_actual_y = tile_bottom_y;
                                cartesian_bottom_y = map_pixel_height - clamped_next_actual_y - player_height;

                                self.velocity_y = 0.0;
                                self.is_airborne = true;
                                clamped = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Apply vertical position update after collision resolution
        self.actual_position.y = map_pixel_height - cartesian_bottom_y - player_height;

        // --- Horizontal movement with collision ---

        let next_x = self.actual_position.x + self.velocity_x;
        let player_top = self.actual_position.y;
        let player_bottom = self.actual_position.y + player_height;

        let cartesian_player_top = map_pixel_height - player_top;
        let cartesian_player_bottom = map_pixel_height - player_bottom;

        let tile_top_y = (cartesian_player_top / TILE_SIZE).floor() as i32;
        let tile_bottom_y = (cartesian_player_bottom / TILE_SIZE).floor() as i32;

        let tile_left_x = (next_x / TILE_SIZE).floor() as i32;
        let tile_right_x = ((next_x + player_width - 1.0) / TILE_SIZE).floor() as i32;

        let mut blocked = false;

        if self.velocity_x > 0.0 {
            let check_x = tile_right_x;

            for tile_y in tile_bottom_y..=tile_top_y {
                if tile_y >= 0 && tile_y < map_height as i32 && check_x >= 0 && check_x < map.width as i32 {
                    if let Some(tile) = map.get_tile(check_x as usize, tile_y as usize) {
                        if tile.is_solid {
                            let tile_top_px = map_pixel_height - (tile_y as f32) * TILE_SIZE;
                            let tile_bottom_px = tile_top_px - TILE_SIZE;

                            let vertical_overlap = !(player_bottom <= tile_bottom_px || player_top >= tile_top_px);

                            if vertical_overlap {
                                blocked = true;
                                break;
                            }
                        }
                    }
                }
            }

            if blocked {
                self.actual_position.x = (check_x as f32 * TILE_SIZE) - player_width;
                self.velocity_x = 0.0;
            } else {
                self.actual_position.x = next_x;
            }
        } else if self.velocity_x < 0.0 {
            let check_x = tile_left_x;

            for tile_y in tile_bottom_y..=tile_top_y {
                if tile_y >= 0 && tile_y < map_height as i32 && check_x >= 0 && check_x < map.width as i32 {
                    if let Some(tile) = map.get_tile(check_x as usize, tile_y as usize) {
                        if tile.is_solid {
                            let tile_top_px = map_pixel_height - (tile_y as f32) * TILE_SIZE;
                            let tile_bottom_px = tile_top_px - TILE_SIZE;

                            let vertical_overlap = !(player_bottom <= tile_bottom_px || player_top >= tile_top_px);

                            if vertical_overlap {
                                blocked = true;
                                break;
                            }
                        }
                    }
                }
            }

            if blocked {
                self.actual_position.x = (check_x as f32 + 1.0) * TILE_SIZE;
                self.velocity_x = 0.0;
            } else {
                self.actual_position.x = next_x;
            }
        }

        // Clamp horizontal position to map bounds
        let max_x = (map.width as f32 * TILE_SIZE) - player_width;
        self.actual_position.x = self.actual_position.x.clamp(0.0, max_x);
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
            true
        } else {
            false
        }
    }
}