use core::input;
use core::constants::*;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec2,
}

impl Camera {
    pub fn update_camera(&self) {
        let cam_x = self.position.x as f32 + TILE_SIZE / 2.0;

        // Offset the camera upwards
        let vertical_offset = screen_height() / 2.0;
        let cam_y = self.position.y + TILE_SIZE / 2.0 - vertical_offset;

        let camera = Camera2D {
            target: vec2(cam_x, cam_y),
            zoom: vec2(1.2 / screen_width(), 1.2 / screen_height()),
            ..Default::default()
        };

        set_camera(&camera);
    }

    pub fn move_camera(&mut self) {
        let speed = 4.0; // pixels per frame
        let input = input::get_omni_input(); // returns Vec2 (e.g. (1, 0))
        self.position += input * speed;
    }
}