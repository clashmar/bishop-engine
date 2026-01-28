use macroquad::prelude::*;
use crate::{
    draw_text_ui, measure_text_ui, is_dropdown_open,
    FIELD_BACKGROUND_COLOR, OUTLINE_COLOR, FIELD_TEXT_COLOR, FIELD_TEXT_SIZE_16,
    HOVER_COLOR, HOVER_COLOR_PLAIN,
};

pub enum ButtonStyle {
    Default,
    Plain,
}

pub fn gui_button(rect: Rect, label: &str, blocked: bool) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Default, FIELD_TEXT_COLOR, Vec2::ZERO, HOVER_COLOR, blocked)
}

pub fn gui_button_plain_default(rect: Rect, label: &str, text_color: Color, blocked: bool) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Plain, text_color, Vec2::ZERO, HOVER_COLOR_PLAIN, blocked)
}

pub fn gui_button_plain_hover(rect: Rect, label: &str, text_color: Color, hover_color: Color, blocked: bool) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Plain, text_color, Vec2::ZERO, hover_color, blocked)
}

pub fn gui_button_y_offset(rect: Rect, label: &str, text_offset: Vec2, blocked: bool) -> bool {
    gui_button_impl(rect, label, ButtonStyle::Default, FIELD_TEXT_COLOR, text_offset, HOVER_COLOR, blocked)
}

fn gui_button_impl(
    rect: Rect,
    label: &str,
    style: ButtonStyle,
    text_color: Color,
    text_offset: Vec2,
    hover_color: Color,
    blocked: bool,
) -> bool {
    let mouse = mouse_position();
    let hovered = rect.contains(vec2(mouse.0, mouse.1));

    let txt_dims = measure_text_ui(label, FIELD_TEXT_SIZE_16, 1.0);
    let txt_y = rect.y + rect.h * 0.7;
    let txt_x = rect.x + (rect.w - txt_dims.width) / 2.;

    match style {
        ButtonStyle::Default => {
            let hovered = rect.contains(vec2(mouse.0, mouse.1));
            let background = if hovered && !is_dropdown_open() && !blocked && !is_mouse_button_down(MouseButton::Left) {
                hover_color
            } else {
                FIELD_BACKGROUND_COLOR
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2., OUTLINE_COLOR);
        }
        ButtonStyle::Plain => {
            if hovered && !is_dropdown_open() && !blocked && !is_mouse_button_down(MouseButton::Left) {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    hover_color,
                );
            }
        }
    }

    draw_text_ui(label, txt_x + text_offset.x, txt_y + text_offset.y, FIELD_TEXT_SIZE_16, text_color);

    is_mouse_button_pressed(MouseButton::Left)
    && hovered
    && !blocked
    && !is_dropdown_open()
}
