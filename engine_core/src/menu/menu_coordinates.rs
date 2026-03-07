use bishop::prelude::*;
use crate::prelude::*;

/// Computes the 16:9 canvas rect fitted into the available screen space.
pub fn compute_canvas_rect(screen_width: f32, screen_height: f32) -> Rect {
    let aspect = DESIGN_RESOLUTION_WIDTH / DESIGN_RESOLUTION_HEIGHT;
    let available_w = screen_width / 1.5;
    let available_h = screen_height - 40.0;

    let (canvas_w, canvas_h) = if available_w / available_h > aspect {
        (available_h * aspect, available_h)
    } else {
        (available_w, available_w / aspect)
    };

    Rect::new(
        (screen_width - canvas_w) / 2.0,
        (screen_height - canvas_h) / 2.0,
        canvas_w,
        canvas_h,
    )
}

/// Converts a screen-space position to normalized (0-1) coordinates.
pub fn screen_to_normalized(screen_pos: Vec2, canvas_origin: Vec2, canvas_size: Vec2) -> Vec2 {
    (screen_pos - canvas_origin) / canvas_size
}

/// Converts a normalized (0-1) position to screen-space coordinates.
pub fn normalized_to_screen(norm_pos: Vec2, canvas_origin: Vec2, canvas_size: Vec2) -> Vec2 {
    norm_pos * canvas_size + canvas_origin
}

/// Converts a normalized rect to a screen-space rect.
pub fn normalized_rect_to_screen(rect: Rect, canvas_origin: Vec2, canvas_size: Vec2) -> Rect {
    Rect::new(
        rect.x * canvas_size.x + canvas_origin.x,
        rect.y * canvas_size.y + canvas_origin.y,
        rect.w * canvas_size.x,
        rect.h * canvas_size.y,
    )
}
