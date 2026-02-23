//! Trait implementations for MacroquadContext.

use super::context::MacroquadContext;
use crate::camera::{Camera, Camera2D};
use crate::draw::{Draw, DrawTexture, DrawTextureParams};
use crate::input::{Input, KeyCode, MouseButton};
use crate::text::{Text, TextDimensions};
use crate::time::Time;
use crate::types::{Color, Vec2};
use crate::window::Window;
use macroquad::prelude as mq;

impl Input for MacroquadContext {
    fn is_key_down(&self, key: KeyCode) -> bool {
        mq::is_key_down(key.into())
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        mq::is_key_pressed(key.into())
    }

    fn is_key_released(&self, key: KeyCode) -> bool {
        mq::is_key_released(key.into())
    }

    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        mq::is_mouse_button_down(button.into())
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        mq::is_mouse_button_pressed(button.into())
    }

    fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        mq::is_mouse_button_released(button.into())
    }

    fn mouse_position(&self) -> (f32, f32) {
        mq::mouse_position()
    }

    fn mouse_wheel(&self) -> (f32, f32) {
        mq::mouse_wheel()
    }

    fn chars_pressed(&self) -> Vec<char> {
        self.char_buffer.clone()
    }

    fn get_time(&self) -> f64 {
        mq::get_time()
    }
}

impl Draw for MacroquadContext {
    fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        mq::draw_rectangle(x, y, w, h, color.into());
    }

    fn draw_rectangle_lines(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        mq::draw_rectangle_lines(x, y, w, h, thickness, color.into());
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        mq::draw_line(x1, y1, x2, y2, thickness, color.into());
    }

    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        mq::draw_circle(x, y, radius, color.into());
    }

    fn draw_circle_lines(&mut self, x: f32, y: f32, radius: f32, thickness: f32, color: Color) {
        mq::draw_circle_lines(x, y, radius, thickness, color.into());
    }

    fn draw_triangle(&mut self, v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
        mq::draw_triangle(
            (v1.x, v1.y).into(),
            (v2.x, v2.y).into(),
            (v3.x, v3.y).into(),
            color.into(),
        );
    }

    fn clear_background(&mut self, color: Color) {
        mq::clear_background(color.into());
    }
}

impl Text for MacroquadContext {
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
        let dims = mq::measure_text(text, None, font_size as u16, 1.0);
        mq::draw_text(text, x, y, font_size, color.into());
        TextDimensions {
            width: dims.width,
            height: dims.height,
            offset_y: dims.offset_y,
        }
    }

    fn measure_text(&self, text: &str, font_size: f32) -> TextDimensions {
        let dims = mq::measure_text(text, None, font_size as u16, 1.0);
        TextDimensions {
            width: dims.width,
            height: dims.height,
            offset_y: dims.offset_y,
        }
    }
}

impl DrawTexture for MacroquadContext {
    type Texture = mq::Texture2D;

    fn draw_texture(&mut self, texture: &Self::Texture, x: f32, y: f32, color: Color) {
        mq::draw_texture(texture, x, y, color.into());
    }

    fn draw_texture_ex(
        &mut self,
        texture: &Self::Texture,
        x: f32,
        y: f32,
        color: Color,
        params: DrawTextureParams,
    ) {
        mq::draw_texture_ex(
            texture,
            x,
            y,
            color.into(),
            mq::DrawTextureParams {
                dest_size: params.dest_size.map(|v| (v.x, v.y).into()),
                source: params.source.map(|r| r.into()),
                rotation: params.rotation,
                flip_x: params.flip_x,
                flip_y: params.flip_y,
                pivot: params.pivot.map(|v| (v.x, v.y).into()),
            },
        );
    }
}

impl Camera for MacroquadContext {
    fn set_camera(&mut self, camera: &Camera2D) {
        mq::set_camera(&mq::Camera2D::from(camera));
    }

    fn set_default_camera(&mut self) {
        mq::set_default_camera();
    }

    fn screen_to_world(&self, camera: &Camera2D, screen_pos: Vec2) -> Vec2 {
        let mq_cam = mq::Camera2D::from(camera);
        let mq_world: mq::Vec2 = mq_cam.screen_to_world((screen_pos.x, screen_pos.y).into());
        Vec2::new(mq_world.x, mq_world.y)
    }
}

impl Window for MacroquadContext {
    fn screen_width(&self) -> f32 {
        mq::screen_width()
    }

    fn screen_height(&self) -> f32 {
        mq::screen_height()
    }
}

impl Time for MacroquadContext {
    fn get_frame_time(&self) -> f32 {
        mq::get_frame_time()
    }

    fn update(&mut self) {
        self.char_buffer.clear();
        while let Some(c) = mq::get_char_pressed() {
            self.char_buffer.push(c);
        }
    }
}
