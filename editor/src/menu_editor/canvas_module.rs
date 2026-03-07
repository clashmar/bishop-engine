// editor/src/menu_editor/canvas_module.rs
use crate::menu_editor::MenuEditor;
use engine_core::prelude::*;
use bishop::prelude::*;

/// Visual canvas for composing menu layouts.
pub struct CanvasModule {
    dragging_element: Option<usize>,
    drag_offset: Vec2,
}

impl CanvasModule {
    /// Creates a new canvas module.
    pub fn new() -> Self {
        Self {
            dragging_element: None,
            drag_offset: Vec2::ZERO,
        }
    }

    /// Updates the canvas and handles input.
    pub fn update(&mut self, ctx: &mut WgpuContext, rect: Rect, menu_editor: &mut MenuEditor, blocked: bool) {
        let mouse: Vec2 = ctx.mouse_position().into();
        let mouse_in_canvas = rect.contains(mouse);

        // Handle Delete key to remove selected element
        if !blocked && ctx.is_key_pressed(KeyCode::Delete) || ctx.is_key_pressed(KeyCode::Backspace) {
            if menu_editor.selected_element_index.is_some() {
                menu_editor.delete_selected_element();
                return;
            }
        }

        if blocked || !mouse_in_canvas {
            return;
        }

        // Handle adding pending element on canvas click
        if let Some(kind) = menu_editor.pending_element_type.take() {
            if ctx.is_mouse_button_pressed(MouseButton::Left) {
                let position = Vec2::new(mouse.x, mouse.y);
                menu_editor.add_element(kind, position);
                return;
            } else {
                menu_editor.pending_element_type = Some(kind);
            }
        }

        // Handle element selection
        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            let clicked_element = menu_editor.current_template().and_then(|template| {
                for (i, element) in template.elements.iter().enumerate().rev() {
                    if element.rect.contains(mouse) {
                        return Some((i, element.rect));
                    }
                }
                None
            });

            if let Some((i, element_rect)) = clicked_element {
                menu_editor.selected_element_index = Some(i);
                self.dragging_element = Some(i);
                self.drag_offset = mouse - vec2(element_rect.x, element_rect.y);
            } else {
                menu_editor.selected_element_index = None;
            }
        }

        // Handle dragging
        if let Some(index) = self.dragging_element {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                if let Some(template) = menu_editor.current_template_mut() {
                    if let Some(element) = template.elements.get_mut(index) {
                        let new_pos = mouse - self.drag_offset;
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
    pub fn draw(&self, ctx: &mut WgpuContext, rect: Rect, menu_editor: &MenuEditor) {
        ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.15, 0.15, 0.2, 1.0));

        ctx.draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, Color::new(0.4, 0.4, 0.4, 1.0));

        // Draw "Menu Canvas" watermark if no template
        if menu_editor.current_template().is_none() {
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

        if let Some(template) = menu_editor.current_template() {
            for (i, element) in template.elements.iter().enumerate() {
                let is_selected = menu_editor.selected_element_index == Some(i);
                self.draw_element(ctx, element, is_selected);
            }

            // Draw "add element" cursor if pending
            if menu_editor.pending_element_type.is_some() {
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

    fn draw_element(&self, ctx: &mut WgpuContext, element: &MenuElement, is_selected: bool) {
        let bg_color = if is_selected {
            Color::new(0.4, 0.5, 0.7, 0.8)
        } else {
            Color::new(0.3, 0.3, 0.35, 0.8)
        };

        ctx.draw_rectangle(
            element.rect.x,
            element.rect.y,
            element.rect.w,
            element.rect.h,
            bg_color,
        );

        let outline_color = if is_selected {
            Color::new(0.6, 0.8, 1.0, 1.0)
        } else {
            Color::new(0.5, 0.5, 0.5, 1.0)
        };

        ctx.draw_rectangle_lines(
            element.rect.x,
            element.rect.y,
            element.rect.w,
            element.rect.h,
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
            element.rect.x + 8.0,
            element.rect.y + element.rect.h * 0.6,
            14.0,
            Color::WHITE,
        );

        // Draw resize handles for selected element
        if is_selected {
            self.draw_resize_handles(ctx, element.rect);
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

impl Default for CanvasModule {
    fn default() -> Self {
        Self::new()
    }
}
