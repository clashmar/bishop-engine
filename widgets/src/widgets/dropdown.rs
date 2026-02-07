use macroquad::prelude::*;
use std::cell::RefCell;
use std::fmt::Display;
use crate::*;

thread_local! {
    static DEFERRED_DROPDOWN_RENDERS: RefCell<Vec<Box<dyn FnOnce()>>> =
        RefCell::new(Vec::new());
}

/// Flushes all deferred dropdown list renders.
///
/// Call this after drawing all modules to ensure dropdown lists render on top.
pub fn flush_dropdown_lists() {
    DEFERRED_DROPDOWN_RENDERS.with(|renders| {
        for render_fn in renders.borrow_mut().drain(..) {
            render_fn();
        }
    });
}

/// The visual style of a dropdown.
#[derive(Clone, Copy)]
pub enum DropDownStyle {
    /// Standard dropdown with background and border.
    Default,
    /// Minimal dropdown with no background.
    Plain,
}

/// A dropdown widget using the builder pattern.
pub struct Dropdown<'a, T> {
    id: WidgetId,
    rect: Rect,
    label: &'a str,
    options: &'a [T],
    to_string: Box<dyn Fn(&T) -> String + 'a>,
    style: DropDownStyle,
    text_color: Color,
    y_offset: f32,
    blocked: bool,
}

impl<'a, T: Clone + PartialEq + Display + 'static> Dropdown<'a, T> {
    /// Creates a new dropdown with the given parameters.
    pub fn new(
        id: WidgetId,
        rect: Rect,
        label: &'a str,
        options: &'a [T],
        to_string: impl Fn(&T) -> String + 'a,
    ) -> Self {
        Self {
            id,
            rect,
            label,
            options,
            to_string: Box::new(to_string),
            style: DropDownStyle::Default,
            text_color: WHITE,
            y_offset: 0.0,
            blocked: false,
        }
    }

    /// Sets the dropdown to use the plain style (no background).
    pub fn plain(mut self) -> Self {
        self.style = DropDownStyle::Plain;
        self
    }

    /// Sets the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Sets the vertical offset for the dropdown list.
    pub fn y_offset(mut self, offset: f32) -> Self {
        self.y_offset = offset;
        self
    }

    /// Sets whether the dropdown is blocked from interaction.
    pub fn blocked(mut self, blocked: bool) -> Self {
        self.blocked = blocked;
        self
    }

    /// Draws the dropdown and returns the selected option if one was clicked.
    pub fn show(self) -> Option<T> {
        const MAX_VISIBLE_ROWS: usize = 8;
        const SCROLL_SPEED: f32 = 5.0;
        const W_PADDING: f32 = 8.0;
        const SCROLLBAR_WIDTH: f32 = 6.0;

        let mut state = dropdown_state::get(self.id);

        let prev_state = state.open;
        state.open = false;
        dropdown_state::set(self.id, state);
        update_global_dropdown_flag();

        let button_clicked = match self.style {
            DropDownStyle::Default => {
                Button::new(self.rect, self.label).blocked(self.blocked).show() && !self.blocked
            }
            DropDownStyle::Plain => {
                Button::new(self.rect, self.label)
                    .plain()
                    .text_color(self.text_color)
                    .blocked(self.blocked)
                    .show() && !self.blocked
            }
        };

        state.open = prev_state;
        dropdown_state::set(self.id, state);
        update_global_dropdown_flag();

        if button_clicked {
            consume_click();
            state.open = !state.open;
        }

        let list_is_open = state.open;

        DROPDOWN_OPEN.with(|r| {
            let was = *r.borrow();
            *r.borrow_mut() = was || list_is_open;
        });

        let mut max_opt_width = 0.0_f32;
        for opt in self.options.iter() {
            let txt = (self.to_string)(opt);
            let width = measure_text_ui(&txt, DEFAULT_FONT_SIZE_16, 1.0).width;
            if width > max_opt_width {
                max_opt_width = width;
            }
        }

        let list_width = self.rect.w.max(max_opt_width + 2.0 * W_PADDING + SCROLLBAR_WIDTH);

        let visible_rows = MAX_VISIBLE_ROWS.min(self.options.len());
        let list_rect = Rect::new(
            self.rect.x,
            self.rect.y + self.rect.h + self.y_offset,
            list_width,
            self.rect.h * visible_rows as f32,
        );

        if list_is_open {
            state.rect = list_rect;
        }

        let mut result: Option<T> = None;

        if list_is_open {
            let total_height = self.rect.h * self.options.len() as f32;
            let max_offset = (total_height - list_rect.h).max(0.0);

            let mouse_pos: Vec2 = mouse_position().into();

            if list_rect.contains(mouse_pos) {
                if is_mouse_button_pressed(MouseButton::Left) {
                    consume_click();
                }
                let (_, wheel_y) = mouse_wheel();
                if wheel_y != 0.0 {
                    let delta = wheel_y * SCROLL_SPEED;
                    state.scroll_offset = (state.scroll_offset - delta).clamp(0.0, max_offset);
                }
            }

            for (i, opt) in self.options.iter().enumerate() {
                let entry_y = list_rect.y + i as f32 * self.rect.h;
                let draw_y = entry_y - state.scroll_offset;

                if draw_y + self.rect.h < list_rect.y + self.rect.h
                    || draw_y > list_rect.y + list_rect.h - self.rect.h
                {
                    continue;
                }

                let entry_rect = Rect::new(list_rect.x, draw_y, list_rect.w, self.rect.h);

                let hovered = entry_rect.contains(mouse_pos);
                if hovered && is_mouse_button_pressed(MouseButton::Left) {
                    consume_click();
                    state.open = false;
                    dropdown_state::set(self.id, state);
                    update_global_dropdown_flag();
                    result = Some(opt.clone());
                    break;
                }
            }

            let options_clone: Vec<T> = self.options.to_vec();
            let scroll_offset = state.scroll_offset;
            let row_height = self.rect.h;
            let to_string_clone: Vec<String> = self.options.iter().map(|o| (self.to_string)(o)).collect();

            DEFERRED_DROPDOWN_RENDERS.with(|renders| {
                renders.borrow_mut().push(Box::new(move || {
                    render_dropdown_list(
                        list_rect,
                        row_height,
                        scroll_offset,
                        &options_clone,
                        &to_string_clone,
                    );
                }));
            });
        }

        let mouse_pos: Vec2 = mouse_position().into();
        if is_mouse_button_pressed(MouseButton::Left)
            && !self.rect.contains(mouse_pos)
            && !(state.open && state.rect.contains(mouse_pos))
        {
            state.open = false;
        }

        dropdown_state::set(self.id, state);
        update_global_dropdown_flag();
        result
    }
}

/// Renders the dropdown list (called from deferred queue).
fn render_dropdown_list<T>(
    list_rect: Rect,
    row_height: f32,
    scroll_offset: f32,
    options: &[T],
    labels: &[String],
) {
    draw_rectangle(
        list_rect.x,
        list_rect.y,
        list_rect.w,
        list_rect.h,
        FIELD_BACKGROUND_COLOR,
    );

    let mouse_pos: Vec2 = mouse_position().into();

    for (i, label) in labels.iter().enumerate() {
        let entry_y = list_rect.y + i as f32 * row_height;
        let draw_y = entry_y - scroll_offset;

        if draw_y + row_height < list_rect.y + row_height
            || draw_y > list_rect.y + list_rect.h - row_height
        {
            continue;
        }

        let entry_rect = Rect::new(list_rect.x, draw_y, list_rect.w, row_height);

        let hovered = entry_rect.contains(mouse_pos);
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
            label,
            entry_rect.x + 5.,
            entry_rect.y + entry_rect.h * 0.7,
            DEFAULT_FONT_SIZE_16,
            FIELD_TEXT_COLOR,
        );
    }

    let total_height = row_height * options.len() as f32;
    if total_height > list_rect.h {
        let thumb_h = (list_rect.h / total_height) * list_rect.h;
        let thumb_y =
            list_rect.y + (scroll_offset / (total_height - list_rect.h)) * (list_rect.h - thumb_h);

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

    draw_rectangle_lines(list_rect.x, list_rect.y, list_rect.w, list_rect.h, 2., OUTLINE_COLOR);
}

/// Internal module for managing dropdown state.
pub mod dropdown_state {
    use macroquad::prelude::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::WidgetId;

    thread_local! {
        pub static STATE: RefCell<HashMap<WidgetId, DropState>> =
            RefCell::new(HashMap::new());
    }

    /// The state of a dropdown widget.
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

    /// Gets the state for a dropdown by id.
    pub fn get(key: WidgetId) -> DropState {
        STATE.with(|s| {
            *s.borrow()
                .get(&key)
                .unwrap_or(&DropState::default())
        })
    }

    /// Sets the state for a dropdown by id.
    pub fn set(key: WidgetId, value: DropState) {
        STATE.with(|s| {
            s.borrow_mut().insert(key, value);
        });
    }
}

/// Updates the global flag indicating whether any dropdown is open.
pub fn update_global_dropdown_flag() {
    dropdown_state::STATE.with(|s| {
        let any = s.borrow().values().any(|st| st.open);
        DROPDOWN_OPEN.with(|f| *f.borrow_mut() = any);
    });
}

/// Returns true if the mouse is over any open dropdown list.
pub fn is_mouse_over_dropdown_list() -> bool {
    dropdown_state::STATE.with(|s| {
        let mouse_pos: Vec2 = mouse_position().into();
        s.borrow().values().any(|st| st.open && st.rect.contains(mouse_pos))
    })
}
