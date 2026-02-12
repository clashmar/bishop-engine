// editor/src/editor_camera_controller.rs
use engine_core::{constants::*, world::room::Room};
use macroquad::prelude::*;

pub const ZOOM_STEP_PERCENT: f32 = 0.5;
pub const MIN_ZOOM: f32 = 0.000001;
pub const MAX_ZOOM: f32 = 0.1;

pub struct EditorCameraController;

impl EditorCameraController {
    /// Call this once per frame from any editor that owns a `Camera2D`.
    pub fn update(camera: &mut Camera2D) {
        // Pan
        if is_mouse_button_down(MouseButton::Middle) || is_key_down(KeyCode::LeftShift) {
            let delta = mouse_delta_position();
            camera.target -= delta * 2.0 / camera.zoom;
        }

        // Zoom (mouse wheel) - discrete steps per notch
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let mut scalar = Self::current_scalar(camera);
            let direction = scroll.signum();
            scalar *= 1.0 + direction * ZOOM_STEP_PERCENT;
            scalar = scalar.clamp(MIN_ZOOM, MAX_ZOOM);
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
    pub fn current_scalar(camera: &Camera2D) -> f32 {
        let aspect = screen_width() / screen_height();
        if aspect > 1.0 {
            // Y holds the scalar
            camera.zoom.y
        } else {
            // X holds the scalar
            camera.zoom.x
        }
    }

    // Turn a scalar zoom into a non‑uniform pair that keeps world
    // units square for the current aspect ratio, snapped to integer pixel ratios.
    pub fn apply_aspect(camera: &mut Camera2D, scalar_zoom: f32) {
        let win_w = screen_width();
        let win_h = screen_height();

        // Snap to integer pixel scale based on the smaller dimension
        // scale = screen_size * zoom / 2.0, so zoom = 2.0 * scale / screen_size
        let current_scale = (win_h * scalar_zoom / 2.0).round().max(1.0);
        let snapped_scalar = 2.0 * current_scale / win_h;

        let aspect = win_w / win_h;
        let (zoom_x, zoom_y) = if aspect > 1.0 {
            (snapped_scalar / aspect, snapped_scalar)
        } else {
            (snapped_scalar, snapped_scalar * aspect)
        };
        camera.zoom = vec2(zoom_x, zoom_y);
    }

    /// Returns a camera centered on a room.
    pub fn camera_for_room(room_size: Vec2, room_position: Vec2, grid_size: f32) -> Camera2D {
        let max_dim_px = (room_size * grid_size).max_element() / 1.5;
        let scalar = editor_zoom_factor(grid_size) / max_dim_px;

        let mut camera = Camera2D {
            target: (room_position + (room_size * grid_size) / 2.0),
            ..Default::default()
        };
        Self::apply_aspect(&mut camera, scalar);
        camera
    }

    /// Reset a `Camera2D` so that the whole room fits the screen.
    pub fn reset_room_editor_camera(camera: &mut Camera2D, room: &Room, grid_size: f32) {
        let map_size = vec2(
            room.current_variant().tilemap.width as f32,
            room.current_variant().tilemap.height as f32,
        );
        *camera = Self::camera_for_room(map_size, room.position, grid_size);
    }

    /// Returns a zoom vector that makes the whole `size` fit the screen,
    /// respecting the current aspect ratio (higher = more zoom).
    pub fn zoom_for_size(size: Vec2, zoom_factor: f32, grid_size: f32) -> Vec2 {
        let max_dim_px = size.max_element() / zoom_factor;
        let scalar = editor_zoom_factor(grid_size) / max_dim_px;

        let mut temp = Camera2D::default();
        temp.zoom = vec2(scalar, scalar);
        EditorCameraController::apply_aspect(&mut temp, scalar);
        temp.zoom
    }
}

