// editor/src/menu_editor/menu_canvas.rs
use crate::menu_editor::resize_handle::*;
use crate::menu_editor::MenuEditor;
use engine_core::prelude::*;
use bishop::prelude::*;

impl MenuEditor {
    /// Updates the menu editor and handles input.
    pub fn update_canvas(
        &mut self,
        ctx: &mut WgpuContext,
        camera: &Camera2D,
        rect: Rect,
        blocked: bool,
    ) {
        let raw_mouse: Vec2 = ctx.mouse_position().into();
        let mouse = camera.screen_to_world(raw_mouse, ctx.screen_width(), ctx.screen_height());
        let mouse_in_canvas = rect.contains(mouse);
        let canvas_origin = Vec2::new(rect.x, rect.y);
        let canvas_size = Vec2::new(rect.w, rect.h);

        // Handle Delete key to remove selected element
        if !blocked && ctx.is_key_pressed(KeyCode::Delete) || ctx.is_key_pressed(KeyCode::Backspace) {
            if !input_is_focused() && self.selected_element_index.is_some() {
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

        // Handle active resize drag
        if self.resizing_handle.is_some() {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                let state = self.resizing_handle.as_ref().unwrap();
                let delta = norm_mouse - state.start_mouse;
                let original = state.original_rect;
                let handle = state.handle;
                let index = state.element_index;
                let child_index = state.child_index;
                let new_rect = apply_resize(original, handle, delta);

                if let Some(child_idx) = child_index {
                    let group_origin = self.current_template()
                        .and_then(|t| t.elements.get(index))
                        .map(|e| Vec2::new(e.rect.x, e.rect.y));
                    if let Some(origin) = group_origin {
                        if let Some(template) = self.current_template_mut() {
                            if let Some(element) = template.elements.get_mut(index) {
                                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                                    if let Some(child) = group.children.get_mut(child_idx) {
                                        child.element.rect.x = new_rect.x - origin.x;
                                        child.element.rect.y = new_rect.y - origin.y;
                                        child.element.rect.w = new_rect.w;
                                        child.element.rect.h = new_rect.h;
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(template) = self.current_template_mut() {
                    if let Some(element) = template.elements.get_mut(index) {
                        element.rect = new_rect;
                    }
                }
            } else {
                self.resizing_handle = None;
            }
            return;
        }

        // Detect resize handle click on the selected element or selected child
        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            if let Some(selected_index) = self.selected_element_index {
                if let Some(child_idx) = self.selected_child_index {
                    // Hit test resize handles on the selected child (unmanaged only)
                    let child_info = self.current_template().and_then(|t| {
                        let element = t.elements.get(selected_index)?;
                        if let MenuElementKind::LayoutGroup(group) = &element.kind {
                            let child = group.children.get(child_idx)?;
                            if !child.managed {
                                let resolved = resolve_layout(group, element.rect);
                                let child_norm_rect = resolved.get(child_idx).copied()?;
                                return Some(child_norm_rect);
                            }
                        }
                        None
                    });
                    if let Some(child_norm_rect) = child_info {
                        let child_screen_rect = normalized_rect_to_screen(child_norm_rect, canvas_origin, canvas_size);
                        if let Some(handle) = hit_test_handles(mouse, child_screen_rect) {
                            self.resizing_handle = Some(ResizeHandleState {
                                element_index: selected_index,
                                child_index: Some(child_idx),
                                handle,
                                original_rect: child_norm_rect,
                                start_mouse: norm_mouse,
                            });
                            return;
                        }
                    }
                } else {
                    // Hit test resize handles on the top-level element
                    if let Some(element_rect) = self
                        .current_template()
                        .and_then(|t| t.elements.get(selected_index))
                        .map(|e| e.rect)
                    {
                        let screen_rect = normalized_rect_to_screen(element_rect, canvas_origin, canvas_size);
                        if let Some(handle) = hit_test_handles(mouse, screen_rect) {
                            self.resizing_handle = Some(ResizeHandleState {
                                element_index: selected_index,
                                child_index: None,
                                handle,
                                original_rect: element_rect,
                                start_mouse: norm_mouse,
                            });
                            return;
                        }
                    }
                }
            }
        }

        // Handle element selection
        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            let clicked = self.current_template().and_then(|template| {
                let sorted = template.sorted_element_indices();
                for &i in sorted.iter().rev() {
                    let element = &template.elements[i];
                    // For layout groups, always check children first
                    if let MenuElementKind::LayoutGroup(group) = &element.kind {
                        let resolved = resolve_layout(group, element.rect);
                        for (child_idx, resolved_rect) in resolved.iter().enumerate().rev() {
                            if resolved_rect.contains(norm_mouse) {
                                let is_managed = group.children.get(child_idx)
                                    .map(|c| c.managed)
                                    .unwrap_or(true);
                                return Some((i, element.rect, Some((child_idx, *resolved_rect, is_managed))));
                            }
                        }
                        if element.rect.contains(norm_mouse) {
                            return Some((i, element.rect, None));
                        }
                        continue;
                    }
                    if element.rect.contains(norm_mouse) {
                        return Some((i, element.rect, None));
                    }
                }
                None
            });

            if let Some((group_or_element_idx, element_rect, child_hit)) = clicked {
                self.selected_element_index = Some(group_or_element_idx);

                if let Some((child_idx, child_rect, is_managed)) = child_hit {
                    self.selected_child_index = Some(child_idx);
                    if !is_managed {
                        self.dragging_element = Some(group_or_element_idx);
                        self.drag_offset = norm_mouse - Vec2::new(child_rect.x, child_rect.y);
                    }
                } else {
                    self.selected_child_index = None;
                    self.dragging_element = Some(group_or_element_idx);
                    self.drag_offset = norm_mouse - Vec2::new(element_rect.x, element_rect.y);
                }
            } else {
                self.selected_element_index = None;
                self.selected_child_index = None;
            }
        }

        // Handle dragging
        if let Some(index) = self.dragging_element {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                let drag_offset = self.drag_offset;
                let child_idx = self.selected_child_index;

                if let Some(child_idx) = child_idx {
                    // Drag unmanaged child: position is relative to group origin
                    let group_origin = self.current_template()
                        .and_then(|t| t.elements.get(index))
                        .map(|e| Vec2::new(e.rect.x, e.rect.y));
                    if let Some(origin) = group_origin {
                        let new_abs = norm_mouse - drag_offset;
                        if let Some(template) = self.current_template_mut() {
                            if let Some(element) = template.elements.get_mut(index) {
                                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                                    if let Some(child) = group.children.get_mut(child_idx) {
                                        child.element.rect.x = new_abs.x - origin.x;
                                        child.element.rect.y = new_abs.y - origin.y;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Drag top-level element
                    if let Some(template) = self.current_template_mut() {
                        if let Some(element) = template.elements.get_mut(index) {
                            let new_pos = norm_mouse - drag_offset;
                            element.rect.x = new_pos.x;
                            element.rect.y = new_pos.y;
                        }
                    }
                }
            } else {
                self.dragging_element = None;
            }
        }
    }

    /// Renders the canvas.
    pub fn draw_canvas(&self, ctx: &mut WgpuContext, camera: &Camera2D, rect: Rect) {
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
            let raw_mouse: Vec2 = ctx.mouse_position().into();
            let world_mouse = camera.screen_to_world(raw_mouse, ctx.screen_width(), ctx.screen_height());
            let sorted = template.sorted_element_indices();
            for i in sorted {
                let element = &template.elements[i];
                let is_selected = self.selected_element_index == Some(i);
                let element_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                self.draw_element(ctx, element, element_rect, canvas_origin, canvas_size, is_selected, true, world_mouse);
            }

            // Draw "add element" cursor if pending
            if self.pending_element_type.is_some() {
                if rect.contains(world_mouse) {
                    ctx.draw_rectangle_lines(
                        world_mouse.x,
                        world_mouse.y,
                        100.0,
                        32.0,
                        2.0,
                        Color::new(0.5, 0.8, 0.5, 0.8),
                    );
                    ctx.draw_text(
                        "Click to place",
                        world_mouse.x + 4.0,
                        world_mouse.y + 20.0,
                        12.0,
                        Color::new(0.5, 0.8, 0.5, 1.0),
                    );
                }
            }
        }
    }

    fn draw_element(
        &self,
        ctx: &mut WgpuContext,
        element: &MenuElement,
        element_rect: Rect,
        canvas_origin: Vec2,
        canvas_size: Vec2,
        is_selected: bool,
        allow_resize: bool,
        world_mouse: Vec2,
    ) {
        match &element.kind {
            MenuElementKind::Button(button) => {
                Button::new(element_rect, &button.text)
                    .font_size(button.font_size)
                    .mouse_position(world_mouse)
                    .show(ctx);

                if is_selected {
                    ctx.draw_rectangle_lines(
                        element_rect.x,
                        element_rect.y,
                        element_rect.w,
                        element_rect.h,
                        2.0,
                        Color::new(0.6, 0.8, 1.0, 1.0),
                    );
                }
            }
            MenuElementKind::LayoutGroup(group) => {
                let has_child_selected = is_selected && self.selected_child_index.is_some();

                let outline_color = if is_selected {
                    Color::new(0.6, 0.9, 0.6, 1.0)
                } else {
                    Color::new(0.4, 0.7, 0.4, 0.8)
                };

                ctx.draw_rectangle_lines(
                    element_rect.x,
                    element_rect.y,
                    element_rect.w,
                    element_rect.h,
                    if is_selected { 2.0 } else { 1.0 },
                    outline_color,
                );

                // Label
                ctx.draw_text(
                    "[Layout Group]",
                    element_rect.x + 4.0,
                    element_rect.y + 12.0,
                    10.0,
                    outline_color,
                );

                // Draw children at resolved positions
                let resolved = resolve_layout(group, element.rect);
                for (child_idx, (child, resolved_rect)) in group.children.iter().zip(resolved.iter()).enumerate() {
                    let child_screen = normalized_rect_to_screen(*resolved_rect, canvas_origin, canvas_size);
                    let is_child_selected = is_selected && self.selected_child_index == Some(child_idx);
                    let child_allow_resize = !child.managed;
                    self.draw_element(ctx, &child.element, child_screen, canvas_origin, canvas_size, is_child_selected, child_allow_resize, world_mouse);
                }

                // Draw resize handles on group only when no child is selected
                if is_selected && !has_child_selected {
                    draw_resize_handles(ctx, element_rect);
                }
                return;
            }
            _ => {
                let bg_color = if is_selected {
                    Color::new(0.4, 0.5, 0.7, 0.8)
                } else {
                    Color::new(0.3, 0.3, 0.35, 0.8)
                };

                ctx.draw_rectangle(
                    element_rect.x,
                    element_rect.y,
                    element_rect.w,
                    element_rect.h,
                    bg_color,
                );

                let outline_color = if is_selected {
                    Color::new(0.6, 0.8, 1.0, 1.0)
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                };

                ctx.draw_rectangle_lines(
                    element_rect.x,
                    element_rect.y,
                    element_rect.w,
                    element_rect.h,
                    if is_selected { 2.0 } else { 1.0 },
                    outline_color,
                );

                let text = match &element.kind {
                    MenuElementKind::Label(label) => label.text.as_str(),
                    MenuElementKind::Button(_) => unreachable!(),
                    MenuElementKind::Panel(_) => "[Panel]",
                    MenuElementKind::LayoutGroup(_) => unreachable!(),
                };

                let text_dims = ctx.measure_text(text, 14.0);
                let text_y = element_rect.y + (element_rect.h - text_dims.height) / 2.0 + text_dims.offset_y;
                ctx.draw_text(
                    text,
                    element_rect.x + 8.0,
                    text_y,
                    14.0,
                    Color::WHITE,
                );
            }
        }

        if is_selected && allow_resize {
            draw_resize_handles(ctx, element_rect);
        }
    }
}
