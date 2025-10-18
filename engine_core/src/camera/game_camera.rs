// engine_core/src/camera/game_camera.rs
use crate::{constants::*, global::tile_size};
use macroquad::prelude::*;

#[derive(Debug)]
pub struct GameCamera {
    pub position: Vec2,
    pub camera: Camera2D,
}

pub fn game_render_target() -> RenderTarget {
    let width = world_virtual_width() as u32;
    let height = world_virtual_height() as u32;
    let rt = render_target(
        width,
        height,
    );
    // Always use Nearest
    rt.texture.set_filter(FilterMode::Nearest);
    rt
}

pub fn world_virtual_width() -> f32 { CAMERA_TILES_X * tile_size() }
pub fn world_virtual_height() -> f32 { CAMERA_TILES_Y * tile_size() }

pub fn zoom_from_scalar(scalar: f32) -> Vec2 {
    // Fixed virtual aspect
    let aspect = world_virtual_width() / world_virtual_height();

    if aspect >= 1.0 {
        vec2(scalar / aspect, scalar)
    } else {
        vec2(scalar, scalar * aspect)
    }
}

impl GameCamera {
    pub fn update_camera(&mut self) {
        // let cam_x = self.position.x as f32 + TILE_SIZE / 2.0;

        // // Offset the camera upwards
        // let vertical_offset = screen_height() / 2.0;
        // let cam_y = self.position.y + TILE_SIZE / 2.0 - vertical_offset;

        // self.camera.target = vec2(cam_x, cam_y);
        // self.camera.zoom = vec2(1.2 / screen_width(), 1.2 / screen_height());

        // set_camera(&self.camera);

    }

    pub fn move_camera(&mut self) {
        // let speed = 4.0; // pixels per frame
        // let input = input::get_omni_input(); // returns Vec2 (e.g. (1, 0))
        // self.position += input * speed;
    }
}

