use std::cell::RefCell;
use std::collections::HashMap;
use crate::*;

/// Draws a horizontal slider widget.
///
/// Returns the new value and whether it changed this frame.
pub fn gui_slider(id: WidgetId, rect: impl Into<Rect>, min: f32, max: f32, value: f32) -> (f32, bool) {
    let rect = rect.into();
    thread_local! {
        static STATE: RefCell<HashMap<WidgetId, (bool, f32)>> =
            RefCell::new(HashMap::new());
    }

    let mut dragging = false;
    let mut drag_offset = 0.0_f32;
    STATE.with(|s| {
        let map = s.borrow();
        if let Some(&(d, off)) = map.get(&id) {
            dragging = d;
            drag_offset = off;
        }
    });

    let track_h = rect.h * 0.2;
    let track_y = rect.y + (rect.h - track_h) * 0.5;
    let handle_sz = rect.h;
    let range = max - min;
    let norm = ((value - min) / range).clamp(0.0, 1.0);
    let handle_x = rect.x + norm * (rect.w - handle_sz);

    macroquad_backend::draw_rectangle(rect.x, rect.y, rect.w, rect.h, FIELD_BACKGROUND_COLOR);
    macroquad_backend::draw_rectangle(rect.x, track_y, rect.w, track_h, Color::new(0.2, 0.2, 0.2, 0.8));
    macroquad_backend::draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., OUTLINE_COLOR);

    let handle_col = if dragging && !is_dropdown_open() {
        Color::new(0.6, 0.6, 0.9, 1.0)
    } else {
        Color::new(0.4, 0.4, 0.8, 1.0)
    };
    macroquad_backend::draw_rectangle(handle_x, rect.y, handle_sz, rect.h, handle_col);
    macroquad_backend::draw_rectangle_lines(handle_x, rect.y, handle_sz, rect.h, 2., Color::WHITE);

    if is_dropdown_open() {
        return (value, false)
    }

    let mouse = macroquad_backend::mouse_position();
    let mouse_vec = Vec2::new(mouse.0, mouse.1);
    let mouse_over_handle = Rect::new(handle_x, rect.y, handle_sz, rect.h)
        .contains(mouse_vec);
    let mouse_over_track = rect.contains(mouse_vec);

    if macroquad_backend::is_mouse_button_pressed(MouseButton::Left) && mouse_over_handle {
        dragging = true;
        drag_offset = mouse.0 - handle_x;
    }

    if macroquad_backend::is_mouse_button_released(MouseButton::Left) {
        dragging = false;
        drag_offset = 0.0;
    }

    let mut new_value = value;
    let mut changed = false;

    if dragging {
        let handle_center = mouse.0 - drag_offset;
        let rel = ((handle_center - rect.x) / (rect.w - handle_sz)).clamp(0.0, 1.0);
        new_value = min + rel * range;
        changed = (new_value - value).abs() > f32::EPSILON;
    } else if mouse_over_track && macroquad_backend::is_mouse_button_pressed(MouseButton::Left) {
        let rel = ((mouse.0 - rect.x) / (rect.w - handle_sz)).clamp(0.0, 1.0);
        new_value = min + rel * range;
        changed = true;
    }

    STATE.with(|s| {
        let mut map = s.borrow_mut();
        map.insert(
            id,
            (dragging, drag_offset),
        );
    });

    (new_value, changed)
}
