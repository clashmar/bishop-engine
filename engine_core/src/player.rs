use crate::{input, physics, tiles::tilemap::TileMap, world::world::GridPos};
use macroquad::{color::Color, math::Vec2, prelude::*};
use crate::{constants::*};

#[derive(Debug, Clone, Copy)]
pub struct PlayerOld {
    pub grid_position: GridPos,
    pub actual_position: Vec2,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub is_airborne: bool,
    pub has_double_jump: bool,   
    pub color: Color,
}

impl PlayerOld {
    pub fn update(&mut self, map: &TileMap) {
        physics::update_physics(self, map);
        self.handle_jump();
        self.handle_horizontal_input();
        self.update_grid_position(map.height);
    }

    pub fn handle_jump(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            if !self.is_airborne {
                // First jump from ground
                self.velocity_y = -10.0;
                self.is_airborne = true;
            } else if self.has_double_jump {
                // Double jump in air
                self.velocity_y = -10.0;
                self.has_double_jump = false;
            }   
        }
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

        let new_grid_pos = GridPos::new(new_grid_x, new_grid_y);

        if new_grid_pos != self.grid_position {
            self.grid_position = new_grid_pos;
            true
        } else {
            false
        }
    }
}