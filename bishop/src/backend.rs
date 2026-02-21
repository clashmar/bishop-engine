//! Global backend functions using thread-local state.
//!
//! This module provides free functions that use thread-local state, similar to macroquad's API.
//! Call `update()` at the start of each frame to capture input state.

#[cfg(feature = "macroquad")]
mod macroquad_backend {
    use crate::*;
    use macroquad::prelude as mq;
    use std::cell::RefCell;

    thread_local! {
        static CHAR_BUFFER: RefCell<Vec<char>> = RefCell::new(Vec::new());
        static FONT: RefCell<Option<mq::Font>> = RefCell::new(None);
    }

    pub async fn next_frame() {
        mq::next_frame().await
    }

    /// Sets the font to use for text rendering.
    pub fn set_font(font: mq::Font) {
        FONT.with(|f| {
            *f.borrow_mut() = Some(font);
        });
    }

    /// Gets a clone of the current font if set.
    pub fn get_font() -> Option<mq::Font> {
        FONT.with(|f| f.borrow().clone())
    }

    /// Updates the input state. Call once per frame before processing input.
    pub fn update() {
        CHAR_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            while let Some(c) = mq::get_char_pressed() {
                buf.push(c);
            }
        });
    }

    /// Returns true if the key is currently held down.
    pub fn is_key_down(key: KeyCode) -> bool {
        mq::is_key_down(key.into())
    }

    /// Returns true if the key was pressed this frame.
    pub fn is_key_pressed(key: KeyCode) -> bool {
        mq::is_key_pressed(key.into())
    }

    /// Returns true if the key was released this frame.
    pub fn is_key_released(key: KeyCode) -> bool {
        mq::is_key_released(key.into())
    }

    /// Returns true if the mouse button is currently held down.
    pub fn is_mouse_button_down(button: MouseButton) -> bool {
        mq::is_mouse_button_down(button.into())
    }

    /// Returns true if the mouse button was pressed this frame.
    pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
        mq::is_mouse_button_pressed(button.into())
    }

    /// Returns true if the mouse button was released this frame.
    pub fn is_mouse_button_released(button: MouseButton) -> bool {
        mq::is_mouse_button_released(button.into())
    }

    /// Returns the current mouse position in screen coordinates.
    pub fn mouse_position() -> (f32, f32) {
        mq::mouse_position()
    }

    /// Returns the mouse wheel scroll delta (horizontal, vertical).
    pub fn mouse_wheel() -> (f32, f32) {
        mq::mouse_wheel()
    }

    pub fn mouse_delta_position() -> (f32, f32) {
        let pos = mq:: mouse_delta_position();
        (pos.x, pos.y)
    }

    /// Returns the time in seconds since the application started.
    pub fn get_time() -> f64 {
        mq::get_time()
    }

    /// Returns characters typed this frame for text input.
    pub fn chars_pressed() -> Vec<char> {
        CHAR_BUFFER.with(|buffer| buffer.borrow().clone())
    }

    /// Consumes and returns the next character pressed, or None if empty.
    pub fn get_char_pressed() -> Option<char> {
        CHAR_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            if buf.is_empty() {
                None
            } else {
                Some(buf.remove(0))
            }
        })
    }

    pub fn get_last_key_pressed() -> Option<KeyCode> {
        match mq::get_last_key_pressed() {
            _ => Some(KeyCode::Unknown),
        }
    }

    /// Draws a filled rectangle.
    pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
        mq::draw_rectangle(x, y, w, h, color.into());
    }

    /// Draws a rectangle outline.
    pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        mq::draw_rectangle_lines(x, y, w, h, thickness, color.into());
    }

    /// Draws a line between two points.
    pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        mq::draw_line(x1, y1, x2, y2, thickness, color.into());
    }

    /// Draws a filled circle.
    pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {
        mq::draw_circle(x, y, radius, color.into());
    }

    /// Draws a circle outline.
    pub fn draw_circle_lines(x: f32, y: f32, radius: f32, thickness: f32, color: Color) {
        mq::draw_circle_lines(x, y, radius, thickness, color.into());
    }

    /// Draws a filled triangle.
    pub fn draw_triangle(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) {
        mq::draw_triangle(
            (v1.x, v1.y).into(),
            (v2.x, v2.y).into(),
            (v3.x, v3.y).into(),
            color.into(),
        );
    }

    /// Clears the screen with the specified color.
    pub fn clear(color: Color) {
        mq::clear_background(color.into());
    }

    /// Draws text at the specified position and returns its dimensions.
    pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
        FONT.with(|f| {
            let font_ref = f.borrow();
            let font = font_ref.as_ref();
            let dims = mq::measure_text(text, font, font_size as u16, 1.0);
            mq::draw_text_ex(
                text,
                x,
                y,
                mq::TextParams {
                    font,
                    font_size: font_size as u16,
                    color: color.into(),
                    ..Default::default()
                },
            );
            TextDimensions {
                width: dims.width,
                height: dims.height,
                offset_y: dims.offset_y,
            }
        })
    }

    pub fn draw_text_ex(
        text: &str,
        x: f32,
        y: f32,
        params: TextParams,
    ) -> TextDimensions {
        FONT.with(|f| {
            let font_ref = f.borrow();
            let font = font_ref.as_ref();

            let dims = mq::measure_text(text, font, params.font_size, params.font_scale);

            let params = mq::TextParams {
                font_size: params.font_size as u16,
                color: mq::BLACK,
                rotation: params.rotation,
                font: params.font,
                ..Default::default()
            };

            mq::draw_text_ex(text, x, y, params);

            TextDimensions {
                width: dims.width,
                height: dims.height,
                offset_y: dims.offset_y,
            }
        })
    }

    /// Measures text without drawing it.
    pub fn measure_text(text: &str, font_size: f32) -> TextDimensions {
        FONT.with(|f| {
            let font_ref = f.borrow();
            let font = font_ref.as_ref();
            let dims = mq::measure_text(text, font, font_size as u16, 1.0);
            TextDimensions {
                width: dims.width,
                height: dims.height,
                offset_y: dims.offset_y,
            }
        })
    }

    /// Initializes the backend with the GNF font.
    pub fn init_with_gnf() {
        crate::font::precache();
        if let Some(font) = crate::font::get_font() {
            set_font(font);
        }
    }

    /// Sets the active camera for rendering.
    pub fn set_camera(camera: &crate::Camera2D) {
        mq::set_camera(&mq::Camera2D::from(camera));
    }

    /// Resets to the default screen-space camera.
    pub fn set_default_camera() {
        mq::set_default_camera();
    }

    /// Converts screen coordinates to world coordinates using the given camera.
    pub fn screen_to_world(camera: &crate::Camera2D, screen_pos: Vec2) -> Vec2 {
        let mq_cam = mq::Camera2D::from(camera);
        let mq_world: mq::Vec2 = mq_cam.screen_to_world((screen_pos.x, screen_pos.y).into());
        Vec2::new(mq_world.x, mq_world.y)
    }

    /// Returns the current screen/window width in pixels.
    pub fn screen_width() -> f32 {
        mq::screen_width()
    }

    /// Returns the current screen/window height in pixels.
    pub fn screen_height() -> f32 {
        mq::screen_height()
    }

    /// Returns the time elapsed since the last frame in seconds.
    pub fn get_frame_time() -> f32 {
        mq::get_frame_time()
    }

    /// Clears the screen with the given color.
    pub fn clear_background(color: Color) {
        mq::clear_background(color.into());
    }

    /// Draws a texture with extended parameters.
    pub fn draw_texture_ex(texture: &mq::Texture2D, x: f32, y: f32, color: Color, params: DrawTextureParams) {
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

    /// Draw a texture.
    pub fn draw_texture(texture: &mq::Texture2D, x: f32, y: f32, color: Color) {
        mq::draw_texture(texture, x, y, color.into());
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_backend::*;
