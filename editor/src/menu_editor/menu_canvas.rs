// editor/src/menu_editor/menu_canvas.rs
use crate::menu_editor::MenuEditor;
use engine_core::prelude::*;
use bishop::prelude::*;

impl MenuEditor {
    /// Updates the menu editor and handles input.
    pub fn update_canvas(
        &mut self,
        ctx: &mut WgpuContext,
        rect: Rect,
        blocked: bool,
    ) {
        let mouse: Vec2 = ctx.mouse_position().into();
        let mouse_in_canvas = rect.contains(mouse);
        let canvas_origin = Vec2::new(rect.x, rect.y);
        let canvas_size = Vec2::new(rect.w, rect.h);

        // Handle Delete key to remove selected element
        if !blocked && ctx.is_key_pressed(KeyCode::Delete) || ctx.is_key_pressed(KeyCode::Backspace) {
            if self.selected_element_index.is_some() {
                self.delete_selected_element();
                return;
            }
        }

        if blocked || !mouse_in_canvas {
            return;
        }

        let norm_mouse = screen_to_normalized(mouse, canvas_origin, canvas_size);

        // Handle adding pending element on canvas click
        if let Some(kind) = self.pending_element_type.take() {
            if ctx.is_mouse_button_pressed(MouseButton::Left) {
                self.add_element(kind, norm_mouse);
                return;
            } else {
                self.pending_element_type = Some(kind);
            }
        }

        // Handle element selection
        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            let clicked_element = self.current_template().and_then(|template| {
                for (i, element) in template.elements.iter().enumerate().rev() {
                    if element.rect.contains(norm_mouse) {
                        return Some((i, element.rect));
                    }
                }
                None
            });

            if let Some((i, element_rect)) = clicked_element {
                self.selected_element_index = Some(i);
                self.dragging_element = Some(i);
                self.drag_offset = norm_mouse - vec2(element_rect.x, element_rect.y);
            } else {
                self.selected_element_index = None;
            }
        }

        // Handle dragging
        if let Some(index) = self.dragging_element {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                let drag_offset = self.drag_offset;
                if let Some(template) = self.current_template_mut() {
                    if let Some(element) = template.elements.get_mut(index) {
                        let new_pos = norm_mouse - drag_offset;
                        element.rect.x = new_pos.x;
                        element.rect.y = new_pos.y;
                    }
                }
            } else {
                self.dragging_element = None;
            }
        }
    }

    /// Renders the canvas.
    pub fn draw_canvas(&self, ctx: &mut WgpuContext, rect: Rect) {
        ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.15, 0.15, 0.2, 1.0));

        ctx.draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, Color::new(0.4, 0.4, 0.4, 1.0));

        let canvas_origin = Vec2::new(rect.x, rect.y);
        let canvas_size = Vec2::new(rect.w, rect.h);

        // Draw "Menu Canvas" watermark if no template
        if self.current_template().is_none() {
            let center_x = rect.x + rect.w * 0.5;
            let center_y = rect.y + rect.h * 0.5;
            ctx.draw_text(
                "No menu selected",
                center_x - 55.0,
                center_y,
                14.0,
                Color::new(0.4, 0.4, 0.4, 1.0),
            );
            return;
        }

        if let Some(template) = self.current_template() {
            for (i, element) in template.elements.iter().enumerate() {
                let is_selected = self.selected_element_index == Some(i);
                let screen_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                self.draw_element(ctx, element, screen_rect, is_selected);
            }

            // Draw "add element" cursor if pending
            if self.pending_element_type.is_some() {
                let mouse: Vec2 = ctx.mouse_position().into();
                if rect.contains(mouse) {
                    ctx.draw_rectangle_lines(
                        mouse.x,
                        mouse.y,
                        100.0,
                        32.0,
                        2.0,
                        Color::new(0.5, 0.8, 0.5, 0.8),
                    );
                    ctx.draw_text(
                        "Click to place",
                        mouse.x + 4.0,
                        mouse.y + 20.0,
                        12.0,
                        Color::new(0.5, 0.8, 0.5, 1.0),
                    );
                }
            }
        }
    }

    fn draw_element(&self, ctx: &mut WgpuContext, element: &MenuElement, screen_rect: Rect, is_selected: bool) {
        let bg_color = if is_selected {
            Color::new(0.4, 0.5, 0.7, 0.8)
        } else {
            Color::new(0.3, 0.3, 0.35, 0.8)
        };

        ctx.draw_rectangle(
            screen_rect.x,
            screen_rect.y,
            screen_rect.w,
            screen_rect.h,
            bg_color,
        );

        let outline_color = if is_selected {
            Color::new(0.6, 0.8, 1.0, 1.0)
        } else {
            Color::new(0.5, 0.5, 0.5, 1.0)
        };

        ctx.draw_rectangle_lines(
            screen_rect.x,
            screen_rect.y,
            screen_rect.w,
            screen_rect.h,
            if is_selected { 2.0 } else { 1.0 },
            outline_color,
        );

        let text = match &element.kind {
            MenuElementKind::Label(label) => label.text.as_str(),
            MenuElementKind::Button(button) => button.text.as_str(),
            MenuElementKind::Spacer(_) => "[Spacer]",
            MenuElementKind::Panel(_) => "[Panel]",
        };

        ctx.draw_text(
            text,
            screen_rect.x + 8.0,
            screen_rect.y + screen_rect.h * 0.6,
            14.0,
            Color::WHITE,
        );

        // Draw resize handles for selected element
        if is_selected {
            self.draw_resize_handles(ctx, screen_rect);
        }
    }

    fn draw_resize_handles(&self, ctx: &mut WgpuContext, rect: Rect) {
        const HANDLE_SIZE: f32 = 6.0;
        const HALF: f32 = HANDLE_SIZE / 2.0;

        let positions = [
            (rect.x - HALF, rect.y - HALF),
            (rect.x + rect.w / 2.0 - HALF, rect.y - HALF),
            (rect.x + rect.w - HALF, rect.y - HALF),
            (rect.x + rect.w - HALF, rect.y + rect.h / 2.0 - HALF),
            (rect.x + rect.w - HALF, rect.y + rect.h - HALF),
            (rect.x + rect.w / 2.0 - HALF, rect.y + rect.h - HALF),
            (rect.x - HALF, rect.y + rect.h - HALF),
            (rect.x - HALF, rect.y + rect.h / 2.0 - HALF),
        ];

        for (x, y) in positions {
            ctx.draw_rectangle(x, y, HANDLE_SIZE, HANDLE_SIZE, Color::WHITE);
            ctx.draw_rectangle_lines(x, y, HANDLE_SIZE, HANDLE_SIZE, 1.0, Color::new(0.3, 0.3, 0.3, 1.0));
        }
    }
}
