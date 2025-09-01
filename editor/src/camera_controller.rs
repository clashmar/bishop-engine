use macroquad::prelude::*;
use core::constants::*;

pub const ZOOM_SPEED_FACTOR: f32 = 0.5;
pub const MIN_ZOOM: f32 = 0.0005;
pub const MAX_ZOOM: f32 = 0.01;

pub struct CameraController;

impl CameraController {
    /// Call this once per frame from any editor that owns a `Camera2D`.
    pub fn update(camera: &mut Camera2D) {
        // Pan (middle‑mouse drag)
        if is_mouse_button_down(MouseButton::Middle) {
            let delta = mouse_delta_position();
            camera.target -= delta / camera.zoom;
        }

        // Zoom (mouse wheel)
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let mut scalar = Self::current_scalar(camera);
            let zoom_speed = ZOOM_SPEED_FACTOR * scalar;
            scalar = (scalar + scroll * zoom_speed).clamp(MIN_ZOOM, MAX_ZOOM);
            Self::apply_aspect(camera, scalar);
        } else {
            let scalar = Self::current_scalar(camera);
            Self::apply_aspect(camera, scalar);
        }
    }

    /// Returns the scalar zoom that would be used for a square window.
    pub fn scalar_zoom(camera: &Camera2D) -> f32 {
        Self::current_scalar(camera)
    }

    // Retrieve the *scalar* zoom that represents the true world‑unit
    // size, regardless of the current aspect ratio.
    fn current_scalar(camera: &Camera2D) -> f32 {
        let aspect = screen_width() / screen_height();
        if aspect > 1.0 {
            // Y holds the scalar
            camera.zoom.y
        } else {
            // X holds the scalar
            camera.zoom.x
        }
    }

    // Rurn a scalar zoom into a non‑uniform pair that keeps world
    // units square for the current aspect ratio.
    fn apply_aspect(camera: &mut Camera2D, scalar_zoom: f32) {
        let aspect = screen_width() / screen_height();
        let (zoom_x, zoom_y) = if aspect > 1.0 {
            // Window wider than tall 
            (scalar_zoom / aspect, scalar_zoom)
        } else {
            // Window taller than wide
            (scalar_zoom, scalar_zoom * aspect)
        };
        camera.zoom = vec2(zoom_x, zoom_y);
    }

    /// Centers the camera on a room.
    pub fn camera_for_room(room_size: Vec2, room_position: Vec2) -> Camera2D {
        let max_dim_px = (room_size * TILE_SIZE).max_element();
        let scalar = WORLD_EDITOR_ZOOM_FACTOR / max_dim_px;

        let aspect = screen_width() / screen_height();
        let (zoom_x, zoom_y) = if aspect > 1.0 {
            (scalar / aspect, scalar)
        } else {
            (scalar, scalar * aspect)
        };

        Camera2D {
            target: (room_position + room_size / 2.0) * TILE_SIZE,
            zoom: vec2(zoom_x, zoom_y),
            ..Default::default()
        }
    }
}

