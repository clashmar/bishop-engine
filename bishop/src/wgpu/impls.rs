//! Trait implementations for WgpuContext.

use super::context::WgpuContext;
use crate::camera::{Camera, Camera2D};
use crate::draw::{Draw, DrawTexture, DrawTextureParams};
use crate::input::{Input, KeyCode, MouseButton};
use crate::text::{Text, TextDimensions};
use crate::time::Time;
use crate::types::{Color, Vec2};
use crate::window::Window;

impl Input for WgpuContext {
    fn is_key_down(&self, key: KeyCode) -> bool {
        self.input.is_key_down(key)
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.input.is_key_pressed(key)
    }

    fn is_key_released(&self, key: KeyCode) -> bool {
        self.input.is_key_released(key)
    }

    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.input.is_mouse_button_down(button)
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.input.is_mouse_button_pressed(button)
    }

    fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.input.is_mouse_button_released(button)
    }

    fn mouse_position(&self) -> (f32, f32) {
        self.input.mouse_position()
    }

    fn mouse_wheel(&self) -> (f32, f32) {
        self.input.mouse_wheel()
    }

    fn chars_pressed(&self) -> Vec<char> {
        self.input.chars_pressed()
    }

    fn get_time(&self) -> f64 {
        self.time.elapsed()
    }
}

impl Draw for WgpuContext {
    fn draw_rectangle(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _color: Color) {
        // Stub: Phase C will implement
    }

    fn draw_rectangle_lines(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _thickness: f32, _color: Color) {
        // Stub: Phase C will implement
    }

    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _thickness: f32, _color: Color) {
        // Stub: Phase C will implement
    }

    fn draw_circle(&mut self, _x: f32, _y: f32, _radius: f32, _color: Color) {
        // Stub: Phase C will implement
    }

    fn draw_circle_lines(&mut self, _x: f32, _y: f32, _radius: f32, _thickness: f32, _color: Color) {
        // Stub: Phase C will implement
    }

    fn draw_triangle(&mut self, _v1: Vec2, _v2: Vec2, _v3: Vec2, _color: Color) {
        // Stub: Phase C will implement
    }

    fn clear(&mut self, color: Color) {
        self.clear_color = Some(color);
    }
}

impl Text for WgpuContext {
    fn draw_text(&mut self, _text: &str, _x: f32, _y: f32, _font_size: f32, _color: Color) -> TextDimensions {
        // Stub: Phase C will implement
        TextDimensions::default()
    }

    fn measure_text(&self, _text: &str, _font_size: f32) -> TextDimensions {
        // Stub: Phase C will implement
        TextDimensions::default()
    }
}

/// Placeholder texture type for wgpu backend.
pub struct WgpuTexture;

impl DrawTexture for WgpuContext {
    type Texture = WgpuTexture;

    fn draw_texture(&mut self, _texture: &Self::Texture, _x: f32, _y: f32, _color: Color) {
        // Stub: Phase C will implement
    }

    fn draw_texture_ex(
        &mut self,
        _texture: &Self::Texture,
        _x: f32,
        _y: f32,
        _color: Color,
        _params: DrawTextureParams,
    ) {
        // Stub: Phase C will implement
    }
}

impl Camera for WgpuContext {
    fn set_camera(&mut self, _camera: &Camera2D) {
        // Stub: Phase C will implement
    }

    fn set_default_camera(&mut self) {
        // Stub: Phase C will implement
    }

    fn screen_to_world(&self, camera: &Camera2D, screen_pos: Vec2) -> Vec2 {
        camera.screen_to_world(screen_pos)
    }
}

impl Window for WgpuContext {
    fn screen_width(&self) -> f32 {
        self.graphics.size.0 as f32
    }

    fn screen_height(&self) -> f32 {
        self.graphics.size.1 as f32
    }
}

impl Time for WgpuContext {
    fn get_frame_time(&self) -> f32 {
        self.time.frame_time()
    }

    fn clear_background(&mut self, color: Color) {
        self.clear_color = Some(color);
    }
}
