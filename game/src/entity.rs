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
        let map_pixel_height = map_height as f32 * TILE_SIZE;

        self.actual_position.x += self.velocity_x;
        // Clamp to map bounds
        let max_x = map.width as f32 * TILE_SIZE - TILE_SIZE;
        self.actual_position.x = self.actual_position.x.clamp(0.0, max_x);

        // Apply gravity to vertical velocity
        self.velocity_y += gravity;

        // Calculate next vertical position based on velocity
        let next_actual_y = (self.actual_position.y + self.velocity_y).clamp(0.0, map_pixel_height - player_height);

        // Convert to cartesian bottom y (bottom-up)
        let mut cartesian_bottom_y = map_pixel_height - next_actual_y - player_height;

        let grid_x = (self.actual_position.x / TILE_SIZE) as usize;
        let grid_y = (cartesian_bottom_y / TILE_SIZE).floor() as i32;

        // Check collision with floor
        if self.velocity_y > 0.0 { // only when falling
            if grid_y >= 0 {
                // Player's left and right edge in tile coordinates
                let left_tile_x = (self.actual_position.x / TILE_SIZE).floor() as i32;
                let right_tile_x = ((self.actual_position.x + TILE_SIZE - 1.0) / TILE_SIZE).floor() as i32;

                // Check both tiles under the player
                let mut on_floor = false;
                for tile_x in left_tile_x..=right_tile_x {
                    if tile_x >= 0 && tile_x < map.width as i32 {
                        if let Some(tile) = map.get_tile(tile_x as usize, grid_y as usize) {
                            if tile.tile_type == TileType::Floor {
                                let tile_top_y = (grid_y as f32 + 1.0) * TILE_SIZE;
                                let prev_cartesian_bottom_y =
                                    map_pixel_height - self.actual_position.y - player_height;

                                if cartesian_bottom_y < tile_top_y
                                    && prev_cartesian_bottom_y >= tile_top_y
                                {
                                    cartesian_bottom_y = tile_top_y;
                                    self.velocity_y = 0.0;
                                    self.is_airborne = false;
                                    self.has_double_jump = true;
                                    on_floor = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                // If not on floor, is_airborne stays true
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