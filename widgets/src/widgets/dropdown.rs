use macroquad::prelude::*;
use std::fmt::Display;
use crate::{
    WidgetId, draw_text_ui, measure_text_ui,
    gui_button, gui_button_plain_default,
    DROPDOWN_OPEN, FIELD_BACKGROUND_COLOR, OUTLINE_COLOR,
    DEFAULT_FONT_SIZE_16, FIELD_TEXT_COLOR,
};

pub enum DropDownStyle {
    Default,
    Plain,
}

pub fn gui_dropdown<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
    blocked: bool,
) -> Option<T> {
    gui_dropdown_impl(
        id,
        rect,
        label,
        options,
        to_string,
        DropDownStyle::Default,
        WHITE,
        0.0,
        blocked,
    )
}

pub fn gui_dropdown_plain<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
    text_color: Color,
    y_offset: f32,
) -> Option<T> {
    gui_dropdown_impl(
        id,
        rect,
        label,
        options,
        to_string,
        DropDownStyle::Plain,
        text_color,
        y_offset,
        false,
    )
}

fn gui_dropdown_impl<T: Clone + PartialEq + Display>(
    id: WidgetId,
    rect: Rect,
    label: &str,
    options: &[T],
    to_string: impl Fn(&T) -> String,
    style: DropDownStyle,
    text_color: Color,
    y_offset: f32,
    blocked: bool,
) -> Option<T> {
    const MAX_VISIBLE_ROWS: usize = 8;
    const SCROLL_SPEED: f32 = 5.0;
    const W_PADDING: f32 = 8.0;
    const SCROLLBAR_WIDTH: f32 = 6.0;

    let mut state = dropdown_state::get(id);

    let prev_state = state.open;
    state.open = false;
    dropdown_state::set(id, state);
    update_global_dropdown_flag();

    let button_clicked = match style {
        DropDownStyle::Default => {
            gui_button(rect, label, blocked) && !blocked
        }
        DropDownStyle::Plain => {
            gui_button_plain_default(rect, label, text_color, blocked) && !blocked
        }
    };

    state.open = prev_state;
    dropdown_state::set(id, state);
    update_global_dropdown_flag();

    if button_clicked {
        state.open = !state.open;
    }

    let list_is_open = state.open;
    state.open = list_is_open;

    let mut any_open = false;
    DROPDOWN_OPEN.with(|r| {
        let was = *r.borrow();
        *r.borrow_mut() = was || list_is_open;
        any_open = *r.borrow();
    });

    let mut max_opt_width = 0.0_f32;
    for opt in options.iter() {
        let txt = to_string(opt);
        let width = measure_text_ui(&txt, DEFAULT_FONT_SIZE_16, 1.0).width;
        if width > max_opt_width {
            max_opt_width = width;
        }
    }

    let list_width = rect.w
        .max(max_opt_width + 2.0 * W_PADDING + SCROLLBAR_WIDTH);

    let visible_rows = MAX_VISIBLE_ROWS.min(options.len());
    let list_rect = Rect::new(
        rect.x,
        rect.y + rect.h + y_offset,
        list_width,
        rect.h * visible_rows as f32,
    );

    if list_is_open {
        state.rect = list_rect;
    }

    if list_is_open {
        let total_height = rect.h * options.len() as f32;
        let max_offset = (total_height - list_rect.h).max(0.0);

        let mouse_pos = mouse_position().into();

        if list_rect.contains(mouse_pos) {
            let (_, wheel_y) = mouse_wheel();
            if wheel_y != 0.0 {
                let delta = wheel_y * SCROLL_SPEED;
                state.scroll_offset = (state.scroll_offset - delta)
                    .clamp(0.0, max_offset);
            }
        }

        draw_rectangle(
            list_rect.x,
            list_rect.y,
            list_rect.w,
            list_rect.h,
            FIELD_BACKGROUND_COLOR,
        );

        for (i, opt) in options.iter().enumerate() {
            let entry_y = list_rect.y + i as f32 * rect.h;

            let draw_y = entry_y - state.scroll_offset;

            if draw_y + rect.h < list_rect.y + rect.h
                || draw_y > list_rect.y + list_rect.h - rect.h
            {
                continue;
            }

            let entry_rect = Rect::new(
                list_rect.x,
                draw_y,
                list_rect.w,
                rect.h,
            );

            let hovered = entry_rect.contains(mouse_pos);
            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                state.open = false;
                dropdown_state::set(id, state);
                update_global_dropdown_flag();
                return Some(opt.clone());
            }

            if hovered {
                draw_rectangle(
                    entry_rect.x,
                    entry_rect.y,
                    entry_rect.w,
                    entry_rect.h,
                    Color::new(0.2, 0.2, 0.2, 0.9),
                );
            }

            draw_text_ui(
                &to_string(opt),
                entry_rect.x + 5.,
                entry_rect.y + entry_rect.h * 0.7,
                DEFAULT_FONT_SIZE_16,
                FIELD_TEXT_COLOR,
            );

            let total_height = rect.h * options.len() as f32;
            if total_height > list_rect.h {
                let thumb_h = (list_rect.h / total_height) * list_rect.h;
                let thumb_y = list_rect.y + (state.scroll_offset / (total_height - list_rect.h)) * (list_rect.h - thumb_h);

                draw_rectangle(
                    list_rect.x + list_rect.w - 6.,
                    list_rect.y,
                    6.,
                    list_rect.h,
                    Color::new(0.2, 0.2, 0.2, 0.5),
                );
                draw_rectangle(
                    list_rect.x + list_rect.w - 6.,
                    thumb_y,
                    6.,
                    thumb_h,
                    Color::new(0.6, 0.6, 0.6, 0.9),
                );
            }

            draw_rectangle_lines(
                list_rect.x,
                list_rect.y,
                list_rect.w,
                list_rect.h,
                2.,
                OUTLINE_COLOR
            );
        }
    }

    let mouse_pos = mouse_position().into();
    if is_mouse_button_pressed(MouseButton::Left)
        && !rect.contains(mouse_pos)
        && !(state.open && state.rect.contains(mouse_pos))
    {
        state.open = false;
    }

    dropdown_state::set(id, state);
    update_global_dropdown_flag();
    None
}

pub mod dropdown_state {
    use macroquad::prelude::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::WidgetId;

    thread_local! {
        pub static STATE: RefCell<HashMap<WidgetId, DropState>> =
            RefCell::new(HashMap::new());
    }

    #[derive(Clone, Copy)]
    pub struct DropState {
        pub open: bool,
        pub rect: Rect,
        pub scroll_offset: f32,
    }

    impl Default for DropState {
        fn default() -> Self {
            Self {
                open: false,
                rect: Rect::default(),
                scroll_offset: 0.,
            }
        }
    }

    pub fn get(key: WidgetId) -> DropState {
        STATE.with(|s| {
            *s.borrow()
                .get(&key)
                .unwrap_or(&DropState::default())
        })
    }

    pub fn set(key: WidgetId, value: DropState) {
        STATE.with(|s| {
            s.borrow_mut().insert(key, value);
        });
    }
}

pub fn update_global_dropdown_flag() {
    dropdown_state::STATE.with(|s| {
        let any = s.borrow().values().any(|st| st.open);
        DROPDOWN_OPEN.with(|f| *f.borrow_mut() = any);
    });
}
