use core::{constants::*, player::Player, tilemap::{self, TileMap}, tile::GridPos};
use crate::{modes::Mode};
use macroquad::prelude::*;
use crate::camera::Camera;

#[derive(Debug, Clone)]
pub struct GameState {
    map: TileMap,
    player: Player,
    mode: Mode,
    camera: Camera,
}

impl GameState {
    pub fn new() -> Self {
        let start_pos = GridPos::new(0, 10);
        let map = tilemap::get_current_map();

        let player = Player { 
            grid_position: start_pos, 
            actual_position: tilemap::tile_to_world(start_pos, map.height),
            velocity_x: 0.0,
            velocity_y: 0.0,
            is_airborne: false,
            has_double_jump: true,
            color: BLUE ,
        };

        Self {
            map: tilemap::get_current_map(),
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
                self.player.update(&self.map);

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

        // self.map.draw();

        draw_rectangle(
            self.player.actual_position.x,
            self.player.actual_position.y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            self.player.color,
        );
    }
}