use core::{constants::*, map::{self, TileMap}};
use crate::{entity::Entity, modes::Mode};
use macroquad::prelude::*;
use crate::camera::Camera;

#[derive(Debug, Clone)]
pub struct GameState {
    map: TileMap,
    player: Entity,
    mode: Mode,
    camera: Camera,
}

impl GameState {
    pub fn new() -> Self {
        let start_tile = ivec2(0, 10);
        let map = map::get_current_map();
        let player = Entity { 
            grid_position: start_tile, 
            actual_position: map::tile_to_world(start_tile, map.height),
            velocity_x: 0.0,
            velocity_y: 0.0,
            is_airborne: false,
            has_double_jump: true,
            color: BLUE ,
        };

        Self {
            map: map::get_current_map(),
            player: player,
            mode: Mode::Explore,
            camera: Camera { 
                position: player.actual_position,
            },
        }
    }

    pub fn update(&mut self) {
        match self.mode {
            Mode::Explore => {
                if is_key_pressed(KeyCode::Space) {
                    if !self.player.is_airborne {
                        // First jump from ground
                        self.player.velocity_y = -10.0;
                        self.player.is_airborne = true;
                    } else if self.player.has_double_jump {
                        // Double jump in air
                        self.player.velocity_y = -10.0;
                        self.player.has_double_jump = false;
                    }
                }

                // Apply gravity
                self.player.update_physics(&self.map, 0.4, self.map.height);

                // Move left/right
                self.player.handle_horizontal_input();

                // Update grid position
                self.player.update_grid_position(self.map.height);

                // Keep camera locked on player in explore mode
                self.camera.position = self.player.actual_position;
            }
            Mode::Combat => {
                self.camera.move_camera();
            }
        }

        if is_key_pressed(KeyCode::C) {
            self.toggle_mode();
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Explore => Mode::Combat,
            Mode::Combat => Mode::Explore,
        };
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        self.camera.update_camera();

        self.map.draw();

        draw_rectangle(
            self.player.actual_position.x,
            self.player.actual_position.y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            self.player.color,
        );
    }
}