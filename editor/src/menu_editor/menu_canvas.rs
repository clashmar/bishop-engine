// editor/src/menu_editor/menu_canvas.rs
use crate::menu_editor::menu_editor::{ReorderDragState, SnapLine};
use crate::menu_editor::resize_handle::*;
use crate::menu_editor::MenuEditor;
use crate::shared::selection::{rect_from_two_points, rects_intersect, draw_selection_box};
use engine_core::prelude::*;
use bishop::prelude::*;

const SNAP_FRACTIONS: [f32; 7] = [0.0, 0.25, 1.0 / 3.0, 0.5, 2.0 / 3.0, 0.75, 1.0];
const SNAP_THRESHOLD: f32 = 0.02;

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
        let canvas_origin = Vec2::new(rect.x, rect.y);
        let canvas_size = Vec2::new(rect.w, rect.h);
        let norm_mouse = screen_to_normalized(mouse, canvas_origin, canvas_size);
        self.last_norm_mouse = Some(norm_mouse);

        let shift_held = ctx.is_key_down(KeyCode::LeftShift) || ctx.is_key_down(KeyCode::RightShift);

        // Arrow key movement for selected elements
        if !self.selected_element_indices.is_empty()
            && self.dragging_element.is_none()
            && !self.box_select_active
            && !input_is_focused()
        {
            let dir = get_omni_input_pressed(ctx);
            if dir != Vec2::ZERO {
                let step = Vec2::new(dir.x / 1920.0, dir.y / 1080.0);

                if let Some(child_idx) = self.selected_child_index {
                    // Move child's relative position
                    if let Some(parent_idx) = self.primary_selected_index() {
                        if let Some(template) = self.current_template_mut() {
                            if let Some(element) = template.elements.get_mut(parent_idx) {
                                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                                    if let Some(child) = group.children.get_mut(child_idx) {
                                        child.element.rect.x += step.x;
                                        child.element.rect.y += step.y;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Move all selected top-level elements
                    let indices: Vec<usize> = self.selected_element_indices.iter().copied().collect();
                    if let Some(template) = self.current_template_mut() {
                        for &i in &indices {
                            if let Some(element) = template.elements.get_mut(i) {
                                element.rect.x += step.x;
                                element.rect.y += step.y;
                            }
                        }
                    }
                }
            }
        }

        // Handle Delete key to remove selected element(s)
        if !blocked && ctx.is_key_pressed(KeyCode::Delete) || ctx.is_key_pressed(KeyCode::Backspace) {
            if !input_is_focused() && !self.selected_element_indices.is_empty() {
                self.delete_selected_element();
                return;
            }
        }

        if blocked {
            return;
        }

        // Handle adding pending element on canvas click
        if let Some(kind) = self.pending_element_type.take() {
            if ctx.is_key_pressed(KeyCode::Escape) {
                return;
            } else if ctx.is_mouse_button_pressed(MouseButton::Left) {
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
                let center_resize = ctx.is_key_down(KeyCode::LeftControl);
                let new_rect = if center_resize {
                    apply_resize_centered(original, handle, delta)
                } else {
                    apply_resize(original, handle, delta)
                };

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

        // Detect resize handle click on the selected element or selected child (single selection only)
        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            if let Some(selected_index) = self.primary_selected_index() {
                if let Some(child_idx) = self.selected_child_index {
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

        // Handle element selection with shift+click and box select
        if ctx.is_mouse_button_pressed(MouseButton::Left) {
            let clicked = self.current_template().and_then(|template| {
                let sorted = template.sorted_element_indices();
                for &i in sorted.iter().rev() {
                    let element = &template.elements[i];
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

            if let Some((idx, element_rect, child_hit)) = clicked {
                if let Some((child_idx, child_rect, is_managed)) = child_hit {
                    // Child click: always single parent+child selection
                    self.selected_element_indices.clear();
                    self.selected_element_indices.insert(idx);
                    self.selected_child_index = Some(child_idx);
                    if is_managed {
                        self.reorder_drag = Some(ReorderDragState {
                            group_index: idx,
                            child_index: child_idx,
                            drop_target: None,
                        });
                    } else {
                        self.dragging_element = Some(idx);
                        self.drag_offset = norm_mouse - Vec2::new(child_rect.x, child_rect.y);
                        self.drag_start_mouse = norm_mouse;
                    }
                } else if shift_held {
                    // Shift+click on top-level element: toggle in selection
                    self.selected_child_index = None;
                    if self.selected_element_indices.contains(&idx) {
                        self.selected_element_indices.remove(&idx);
                    } else {
                        self.selected_element_indices.insert(idx);
                    }
                } else if self.selected_element_indices.contains(&idx) {
                    // Click on already-selected element: keep selection, start drag
                    self.selected_child_index = None;
                    self.dragging_element = Some(idx);
                    self.drag_offset = norm_mouse - Vec2::new(element_rect.x, element_rect.y);
                    self.drag_start_mouse = norm_mouse;
                    // Store start positions for multi-drag
                    let indices: Vec<usize> = self.selected_element_indices.iter().copied().collect();
                    let start_rects: Vec<(usize, Vec2)> = self.current_template()
                        .map(|t| indices.into_iter()
                            .filter_map(|si| t.elements.get(si).map(|el| (si, Vec2::new(el.rect.x, el.rect.y))))
                            .collect())
                        .unwrap_or_default();
                    self.drag_start_rects = start_rects;
                } else {
                    // Click on unselected element without shift: single select + start drag
                    self.selected_element_indices.clear();
                    self.selected_element_indices.insert(idx);
                    self.selected_child_index = None;
                    self.dragging_element = Some(idx);
                    self.drag_offset = norm_mouse - Vec2::new(element_rect.x, element_rect.y);
                    self.drag_start_mouse = norm_mouse;
                    self.drag_start_rects.clear();
                }
            } else {
                // Clicked on empty space
                if !shift_held {
                    self.selected_element_indices.clear();
                    self.selected_child_index = None;
                }
                // Start box select
                self.box_select_start = Some(norm_mouse);
                self.box_select_active = true;
            }
        }

        // Handle box select drag and release
        if self.box_select_active {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                // Box select is in progress; drawing handled in draw_canvas
                return;
            } else {
                // Mouse released: finalize box select
                if let Some(start) = self.box_select_start.take() {
                    let sel_rect = rect_from_two_points(start, norm_mouse);
                    let matched: Vec<usize> = self.current_template()
                        .map(|t| t.elements.iter().enumerate()
                            .filter(|(_, el)| rects_intersect(sel_rect, el.rect))
                            .map(|(i, _)| i)
                            .collect())
                        .unwrap_or_default();
                    for i in matched {
                        self.selected_element_indices.insert(i);
                    }
                    self.selected_child_index = None;
                }
                self.box_select_active = false;
                return;
            }
        }

        // Handle reorder drag for managed children
        if self.reorder_drag.is_some() {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                let reorder = self.reorder_drag.as_ref().unwrap();
                let group_index = reorder.group_index;
                let child_index = reorder.child_index;
                let drop = self.current_template().and_then(|t| {
                    let element = t.elements.get(group_index)?;
                    if let MenuElementKind::LayoutGroup(group) = &element.kind {
                        compute_reorder_drop_index(group, element.rect, norm_mouse, child_index)
                    } else {
                        None
                    }
                });
                self.reorder_drag.as_mut().unwrap().drop_target = drop;
            } else {
                let reorder = self.reorder_drag.as_ref().unwrap();
                let group_index = reorder.group_index;
                let child_index = reorder.child_index;
                let drop_target = reorder.drop_target;
                self.reorder_drag = None;

                if let Some(target) = drop_target {
                    if target != child_index {
                        if let Some(template) = self.current_template_mut() {
                            if let Some(element) = template.elements.get_mut(group_index) {
                                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                                    if child_index < group.children.len() && target <= group.children.len() {
                                        let child = group.children.remove(child_index);
                                        let effective = if target > child_index {
                                            target - 1
                                        } else {
                                            target
                                        };
                                        let insert_at = effective.min(group.children.len());
                                        group.children.insert(insert_at, child);
                                        self.selected_child_index = Some(insert_at);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle dragging
        if let Some(anchor_index) = self.dragging_element {
            if ctx.is_mouse_button_down(MouseButton::Left) {
                let drag_offset = self.drag_offset;
                let child_idx = self.selected_child_index;

                // Shift-lock: constrain to dominant axis
                let norm_mouse = if shift_held {
                    let delta = norm_mouse - self.drag_start_mouse;
                    if delta.x.abs() > delta.y.abs() {
                        Vec2::new(norm_mouse.x, self.drag_start_mouse.y)
                    } else {
                        Vec2::new(self.drag_start_mouse.x, norm_mouse.y)
                    }
                } else {
                    norm_mouse
                };

                if let Some(child_idx) = child_idx {
                    // Drag unmanaged child: position is relative to group origin
                    let group_origin = self.current_template()
                        .and_then(|t| t.elements.get(anchor_index))
                        .map(|e| Vec2::new(e.rect.x, e.rect.y));
                    if let Some(origin) = group_origin {
                        let new_abs = norm_mouse - drag_offset;
                        if let Some(template) = self.current_template_mut() {
                            if let Some(element) = template.elements.get_mut(anchor_index) {
                                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                                    if let Some(child) = group.children.get_mut(child_idx) {
                                        child.element.rect.x = new_abs.x - origin.x;
                                        child.element.rect.y = new_abs.y - origin.y;
                                    }
                                }
                            }
                        }
                    }
                } else if self.selected_element_indices.len() > 1 && !self.drag_start_rects.is_empty() {
                    // Multi-drag: compute delta from anchor's start position
                    let anchor_start = self.drag_start_rects.iter()
                        .find(|(i, _)| *i == anchor_index)
                        .map(|(_, pos)| *pos);
                    if let Some(anchor_start) = anchor_start {
                        let new_anchor_pos = norm_mouse - drag_offset;
                        let delta = new_anchor_pos - anchor_start;
                        let snapping = ctx.is_key_down(KeyCode::S) && !input_is_focused();

                        let start_rects: Vec<(usize, Vec2)> = self.drag_start_rects.clone();
                        let template_idx = self.current_template_index;
                        if let Some(ti) = template_idx {
                            let mut new_snap_lines = Vec::new();
                            let mut snap_delta = delta;
                            if let Some(template) = self.templates.get_mut(ti) {
                                if snapping {
                                    if let Some(anchor_el) = template.elements.get(anchor_index) {
                                        let anchor_size = Vec2::new(anchor_el.rect.w, anchor_el.rect.h);
                                        let (snapped, lines) = snap_center_to_fractions(anchor_start + delta, anchor_size);
                                        snap_delta = snapped - anchor_start;
                                        new_snap_lines = lines;
                                    }
                                }
                                for &(i, start_pos) in &start_rects {
                                    if let Some(element) = template.elements.get_mut(i) {
                                        let target = start_pos + snap_delta;
                                        element.rect.x = target.x;
                                        element.rect.y = target.y;
                                    }
                                }
                            }
                            self.snap_lines = new_snap_lines;
                        }
                        if !snapping {
                            self.snap_lines.clear();
                        }
                    }
                } else {
                    // Single element drag
                    let snapping = ctx.is_key_down(KeyCode::S) && !input_is_focused();
                    if let Some(template) = self.current_template_mut() {
                        if let Some(element) = template.elements.get_mut(anchor_index) {
                            let new_pos = norm_mouse - drag_offset;
                            if snapping {
                                let size = Vec2::new(element.rect.w, element.rect.h);
                                let (snapped, lines) = snap_center_to_fractions(new_pos, size);
                                element.rect.x = snapped.x;
                                element.rect.y = snapped.y;
                                self.snap_lines = lines;
                            } else {
                                element.rect.x = new_pos.x;
                                element.rect.y = new_pos.y;
                                self.snap_lines.clear();
                            }
                        }
                    }
                }
            } else {
                self.dragging_element = None;
                self.drag_start_rects.clear();
                self.snap_lines.clear();
            }
        }
    }

    /// Renders the menu fullscreen in preview mode without editor overlays.
    pub fn draw_preview_canvas(&self, ctx: &mut WgpuContext, camera: &Camera2D, rect: Rect) {
        let canvas_origin = Vec2::new(rect.x, rect.y);
        let canvas_size = Vec2::new(rect.w, rect.h);

        let Some(template) = self.current_template() else {
            return;
        };

        match template.background {
            MenuBackground::SolidColor(color) => {
                ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
            }
            MenuBackground::Dimmed(alpha) => {
                ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.0, 0.0, 0.0, alpha));
            }
            MenuBackground::None => {}
        }

        let raw_mouse: Vec2 = ctx.mouse_position().into();
        let world_mouse = camera.screen_to_world(raw_mouse, ctx.screen_width(), ctx.screen_height());

        for i in template.sorted_element_indices() {
            let element = &template.elements[i];
            let element_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
            self.draw_element(ctx, element, element_rect, canvas_origin, canvas_size, false, false, world_mouse, true);
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
            // Render background preview
            match template.background {
                MenuBackground::SolidColor(color) => {
                    ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
                }
                MenuBackground::Dimmed(alpha) => {
                    ctx.draw_rectangle(
                        rect.x, rect.y, rect.w, rect.h,
                        Color::new(0.0, 0.0, 0.0, alpha),
                    );
                }
                MenuBackground::None => {}
            }

            // Draw snap guide lines
            let guide_color = Color::new(1.0, 1.0, 0.4, 0.4);
            for line in &self.snap_lines {
                match line {
                    SnapLine::Vertical(nx) => {
                        let screen_x = rect.x + nx * rect.w;
                        ctx.draw_rectangle(screen_x - 0.5, rect.y, 1.0, rect.h, guide_color);
                    }
                    SnapLine::Horizontal(ny) => {
                        let screen_y = rect.y + ny * rect.h;
                        ctx.draw_rectangle(rect.x, screen_y - 0.5, rect.w, 1.0, guide_color);
                    }
                }
            }

            let raw_mouse: Vec2 = ctx.mouse_position().into();
            let world_mouse = camera.screen_to_world(raw_mouse, ctx.screen_width(), ctx.screen_height());
            let sorted = template.sorted_element_indices();
            for i in sorted {
                let element = &template.elements[i];
                let is_selected = self.selected_element_indices.contains(&i);
                let element_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                self.draw_element(ctx, element, element_rect, canvas_origin, canvas_size, is_selected, true, world_mouse, false);
            }

            // Draw placement cursor if pending
            if self.pending_element_type.is_some() {
                if rect.contains(world_mouse) {
                    let size = 32.0;
                    let half = size / 2.0;
                    ctx.draw_rectangle_lines(
                        world_mouse.x - half,
                        world_mouse.y - half,
                        size,
                        size,
                        2.0,
                        Color::new(0.5, 0.8, 0.5, 0.8),
                    );
                }
            }
        }

        // Draw box selection overlay
        if self.box_select_active {
            if let (Some(start), Some(current)) = (self.box_select_start, self.last_norm_mouse) {
                let start_screen = Vec2::new(
                    canvas_origin.x + start.x * canvas_size.x,
                    canvas_origin.y + start.y * canvas_size.y,
                );
                let end_screen = Vec2::new(
                    canvas_origin.x + current.x * canvas_size.x,
                    canvas_origin.y + current.y * canvas_size.y,
                );
                draw_selection_box(ctx, start_screen, end_screen);
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
        preview: bool,
    ) {
        match &element.kind {
            MenuElementKind::Button(button) => {
                let display_text = format!("{}", button.text_key);
                Button::new(element_rect, &display_text)
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

                if let Some(bg) = &group.background {
                    ctx.draw_rectangle(
                        element_rect.x,
                        element_rect.y,
                        element_rect.w,
                        element_rect.h,
                        bg.render_color(),
                    );
                }

                if !preview {
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
                    let group_label = if !element.name.is_empty() {
                        format!("[{}]", element.name)
                    } else {
                        "[Layout Group]".to_string()
                    };
                    ctx.draw_text(
                        &group_label,
                        element_rect.x + 4.0,
                        element_rect.y + 12.0,
                        10.0,
                        outline_color,
                    );
                }

                // Draw children at resolved positions
                let resolved = resolve_layout(group, element.rect);
                let reorder_info = self.reorder_drag.as_ref().filter(|r| {
                    self.selected_element_indices.contains(&r.group_index)
                });
                let dragged_child_idx = reorder_info.map(|r| r.child_index);
                let drop_target = reorder_info.and_then(|r| r.drop_target);

                for (child_idx, (child, resolved_rect)) in group.children.iter().zip(resolved.iter()).enumerate() {
                    let child_screen = normalized_rect_to_screen(*resolved_rect, canvas_origin, canvas_size);
                    let is_child_selected = is_selected && self.selected_child_index == Some(child_idx);
                    let child_allow_resize = !child.managed;

                    // Dim the dragged child at its original slot
                    if dragged_child_idx == Some(child_idx) {
                        ctx.draw_rectangle(
                            child_screen.x, child_screen.y,
                            child_screen.w, child_screen.h,
                            Color::new(0.0, 0.0, 0.0, 0.3),
                        );
                    }

                    self.draw_element(ctx, &child.element, child_screen, canvas_origin, canvas_size, is_child_selected, child_allow_resize, world_mouse, preview);
                }

                // Draw drop indicator line
                if let Some(target) = drop_target {
                    let managed_rects: Vec<(usize, Rect)> = group.children.iter()
                        .zip(resolved.iter())
                        .enumerate()
                        .filter(|(_, (child, _))| child.managed)
                        .map(|(idx, (_, rect))| (idx, *rect))
                        .collect();

                    let indicator_color = Color::new(0.3, 0.7, 1.0, 0.9);
                    let managed_slot = child_index_to_managed_slot(group, target);

                    let spacing_x = group.layout.spacing / 1920.0;
                    let spacing_y = group.layout.spacing / 1080.0;

                    draw_reorder_indicator(
                        ctx, &managed_rects, managed_slot,
                        &group.layout.direction,
                        spacing_x, spacing_y,
                        canvas_origin, canvas_size,
                        indicator_color,
                    );
                }

                // Draw resize handles on group only when no child is selected
                if is_selected && !has_child_selected {
                    draw_resize_handles(ctx, element_rect);
                }
                return;
            }
            MenuElementKind::Label(label) => {
                if !preview {
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
                }

                let text = &label.text_key;
                let text_dims = ctx.measure_text(text, label.font_size);
                let text_x = match label.alignment {
                    HorizontalAlign::Left => element_rect.x,
                    HorizontalAlign::Center => element_rect.x + (element_rect.w - text_dims.width) / 2.0,
                    HorizontalAlign::Right => element_rect.x + element_rect.w - text_dims.width,
                };
                let text_y = element_rect.y + (element_rect.h - text_dims.height) / 2.0 + text_dims.offset_y;
                ctx.draw_text(text, text_x, text_y, label.font_size, label.color);
            }
            MenuElementKind::Panel(panel) => {
                ctx.draw_rectangle(
                    element_rect.x,
                    element_rect.y,
                    element_rect.w,
                    element_rect.h,
                    panel.background.render_color(),
                );

                if !preview {
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

                    let label = if !element.name.is_empty() {
                        element.name.as_str()
                    } else {
                        "[Panel]"
                    };

                    ctx.draw_text(
                        label,
                        element_rect.x + 4.0,
                        element_rect.y + 12.0,
                        10.0,
                        outline_color,
                    );
                }
            }
        }

        if is_selected && allow_resize {
            draw_resize_handles(ctx, element_rect);
        }
    }
}

/// Computes the drop target index (in the full children Vec) from mouse position.
///
/// Compares the mouse against midpoints of managed children to determine
/// where the dragged child should be inserted.
fn compute_reorder_drop_index(
    group: &LayoutGroupElement,
    group_rect: Rect,
    norm_mouse: Vec2,
    dragged_child_index: usize,
) -> Option<usize> {
    let resolved = resolve_layout(group, group_rect);

    // Collect managed children: (vec_index, resolved_rect)
    let managed: Vec<(usize, Rect)> = group.children.iter()
        .zip(resolved.iter())
        .enumerate()
        .filter(|(_, (child, _))| child.managed)
        .map(|(idx, (_, rect))| (idx, *rect))
        .collect();

    if managed.len() < 2 {
        return None;
    }

    let managed_slot = match group.layout.direction {
        LayoutDirection::Vertical => {
            let mut slot = managed.len();
            for (i, (_, rect)) in managed.iter().enumerate() {
                let midpoint = rect.y + rect.h * 0.5;
                if norm_mouse.y < midpoint {
                    slot = i;
                    break;
                }
            }
            slot
        }
        LayoutDirection::Horizontal => {
            let mut slot = managed.len();
            for (i, (_, rect)) in managed.iter().enumerate() {
                let midpoint = rect.x + rect.w * 0.5;
                if norm_mouse.x < midpoint {
                    slot = i;
                    break;
                }
            }
            slot
        }
        LayoutDirection::Grid { columns } => {
            let cols = columns.max(1) as usize;
            if let Some((_, first_rect)) = managed.first() {
                let spacing_x = if managed.len() > 1 && cols > 1 {
                    (managed.get(1).map(|(_, r)| r.x).unwrap_or(first_rect.x) - first_rect.x - first_rect.w).max(0.0)
                } else {
                    0.0
                };
                let spacing_y = if managed.len() > cols {
                    (managed.get(cols).map(|(_, r)| r.y).unwrap_or(first_rect.y) - first_rect.y - first_rect.h).max(0.0)
                } else {
                    0.0
                };

                let cell_w = first_rect.w + spacing_x;
                let cell_h = first_rect.h + spacing_y;

                let rel_x = norm_mouse.x - first_rect.x;
                let rel_y = norm_mouse.y - first_rect.y;

                let col = (rel_x / cell_w).floor().max(0.0) as usize;
                let row = (rel_y / cell_h).floor().max(0.0) as usize;

                let col = col.min(cols - 1);
                let slot = row * cols + col;
                slot.min(managed.len())
            } else {
                0
            }
        }
    };

    // Map managed slot back to Vec index
    let dragged_managed_slot = managed.iter()
        .position(|(idx, _)| *idx == dragged_child_index);

    // If dropping at the same managed slot or right after, no change needed
    if let Some(d_slot) = dragged_managed_slot {
        if managed_slot == d_slot || managed_slot == d_slot + 1 {
            return Some(dragged_child_index);
        }
    }

    // Convert managed slot to Vec index
    let vec_index = if managed_slot >= managed.len() {
        // Insert after the last managed child
        managed.last().map(|(idx, _)| idx + 1).unwrap_or(0)
    } else {
        managed[managed_slot].0
    };

    Some(vec_index)
}

/// Maps a Vec child index to its managed slot index.
fn child_index_to_managed_slot(group: &LayoutGroupElement, child_index: usize) -> usize {
    group.children.iter()
        .take(child_index)
        .filter(|c| c.managed)
        .count()
}

/// Draws a drop indicator line at the target managed slot position.
fn draw_reorder_indicator(
    ctx: &mut WgpuContext,
    managed_rects: &[(usize, Rect)],
    managed_slot: usize,
    direction: &LayoutDirection,
    spacing_x: f32,
    spacing_y: f32,
    canvas_origin: Vec2,
    canvas_size: Vec2,
    color: Color,
) {
    if managed_rects.is_empty() {
        return;
    }

    let thickness = 2.0;

    match direction {
        LayoutDirection::Vertical => {
            let (y, x, w) = if managed_slot == 0 {
                let (_, first) = &managed_rects[0];
                (first.y - spacing_y * 0.5, first.x, first.w)
            } else if managed_slot >= managed_rects.len() {
                let (_, last) = managed_rects.last().unwrap();
                (last.y + last.h + spacing_y * 0.5, last.x, last.w)
            } else {
                let (_, prev) = &managed_rects[managed_slot - 1];
                let (_, next) = &managed_rects[managed_slot];
                let mid_y = (prev.y + prev.h + next.y) * 0.5;
                (mid_y, next.x, next.w)
            };
            let screen = normalized_rect_to_screen(
                Rect::new(x, y - 0.001, w, 0.002),
                canvas_origin, canvas_size,
            );
            ctx.draw_rectangle(screen.x, screen.y, screen.w, thickness, color);
        }
        LayoutDirection::Horizontal => {
            let (x, y, h) = if managed_slot == 0 {
                let (_, first) = &managed_rects[0];
                (first.x - spacing_x * 0.5, first.y, first.h)
            } else if managed_slot >= managed_rects.len() {
                let (_, last) = managed_rects.last().unwrap();
                (last.x + last.w + spacing_x * 0.5, last.y, last.h)
            } else {
                let (_, prev) = &managed_rects[managed_slot - 1];
                let (_, next) = &managed_rects[managed_slot];
                let mid_x = (prev.x + prev.w + next.x) * 0.5;
                (mid_x, next.y, next.h)
            };
            let screen = normalized_rect_to_screen(
                Rect::new(x - 0.001, y, 0.002, h),
                canvas_origin, canvas_size,
            );
            ctx.draw_rectangle(screen.x, screen.y, thickness, screen.h, color);
        }
        LayoutDirection::Grid { .. } => {
            let (y, x, w) = if managed_slot == 0 {
                let (_, first) = &managed_rects[0];
                (first.y - spacing_y * 0.5, first.x, first.w)
            } else if managed_slot >= managed_rects.len() {
                let (_, last) = managed_rects.last().unwrap();
                (last.y + last.h + spacing_y * 0.5, last.x, last.w)
            } else {
                let (_, prev) = &managed_rects[managed_slot - 1];
                let (_, next) = &managed_rects[managed_slot];
                let mid_y = (prev.y + prev.h + next.y) * 0.5;
                (mid_y, next.x, next.w)
            };
            let screen = normalized_rect_to_screen(
                Rect::new(x, y - 0.001, w, 0.002),
                canvas_origin, canvas_size,
            );
            ctx.draw_rectangle(screen.x, screen.y, screen.w, thickness, color);
        }
    }
}

/// Snaps an element's center to common fractional positions on each axis.
///
/// Returns the adjusted top-left position and any active snap guide lines.
fn snap_center_to_fractions(pos: Vec2, size: Vec2) -> (Vec2, Vec<SnapLine>) {
    let center = pos + size * 0.5;
    let mut result = pos;
    let mut lines = Vec::new();

    if let Some(snapped_x) = nearest_snap(center.x) {
        result.x = snapped_x - size.x * 0.5;
        lines.push(SnapLine::Vertical(snapped_x));
    }
    if let Some(snapped_y) = nearest_snap(center.y) {
        result.y = snapped_y - size.y * 0.5;
        lines.push(SnapLine::Horizontal(snapped_y));
    }

    (result, lines)
}

/// Returns the nearest snap fraction if within threshold, or None.
fn nearest_snap(value: f32) -> Option<f32> {
    let mut best: Option<(f32, f32)> = None;
    for &frac in &SNAP_FRACTIONS {
        let dist = (value - frac).abs();
        if dist < SNAP_THRESHOLD {
            if best.map_or(true, |(_, d)| dist < d) {
                best = Some((frac, dist));
            }
        }
    }
    best.map(|(frac, _)| frac)
}
