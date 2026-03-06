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

    /// Updates the canvas.
    pub fn update(&mut self, ctx: &mut WgpuContext, rect: Rect, menu_editor: &mut MenuEditor, blocked: bool) {
        let mouse: Vec2 = ctx.mouse_position().into();
        let mouse_in_canvas = rect.contains(mouse);

        if blocked || !mouse_in_canvas {
            return;
        }

        if let Some(template) = &mut menu_editor.current_template {
            if ctx.is_mouse_button_pressed(MouseButton::Left) {
                for (i, element) in template.elements.iter().enumerate() {
                    if element.rect.contains(mouse) {
                        menu_editor.selected_element_index = Some(i);
                        self.dragging_element = Some(i);
                        self.drag_offset = mouse - vec2(element.rect.x, element.rect.y);
                        break;
                    }
                }
            }

            if let Some(index) = self.dragging_element {
                if ctx.is_mouse_button_down(MouseButton::Left) {
                    if let Some(element) = template.elements.get_mut(index) {
                        let new_pos = mouse - self.drag_offset;
                        element.rect.x = new_pos.x;
                        element.rect.y = new_pos.y;
                    }
                } else {
                    self.dragging_element = None;
                }
            }
        }
    }

    /// Renders the canvas.
    pub fn draw(&self, ctx: &mut WgpuContext, rect: Rect, menu_editor: &MenuEditor) {
        ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.15, 0.15, 0.2, 1.0));

        ctx.draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, Color::new(0.4, 0.4, 0.4, 1.0));

        if let Some(template) = &menu_editor.current_template {
            for (i, element) in template.elements.iter().enumerate() {
                let is_selected = menu_editor.selected_element_index == Some(i);
                self.draw_element(ctx, element, is_selected);
            }
        }

        let center_x = rect.x + rect.w * 0.5;
        let center_y = rect.y + rect.h * 0.5;
        ctx.draw_text(
            "Menu Canvas",
            center_x - 45.0,
            center_y,
            14.0,
            Color::new(0.4, 0.4, 0.4, 1.0),
        );
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
            MenuElementKind::Label(label) => &label.text,
            MenuElementKind::Button(button) => &button.text,
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
    }
}

impl Default for CanvasModule {
    fn default() -> Self {
        Self::new()
    }
}
