// editor/src/menu_editor/menu_properties_panel/layout_properties.rs
use crate::menu_editor::MenuEditor;
use super::{ROW_HEIGHT, LABEL_WIDTH, FIELD_HEIGHT};
use engine_core::prelude::*;
use bishop::prelude::*;

impl MenuEditor {
    pub(super) fn draw_layout_group_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
    ) {
        let (direction, grid_cols, spacing, padding, h_align, v_align, item_w, item_h, child_count, nav_up, nav_down, nav_left, nav_right) = {
            let Some(element) = self.selected_element() else { return };
            let MenuElementKind::LayoutGroup(group) = &element.kind else { return };
            let cols = match group.layout.direction {
                LayoutDirection::Grid { columns } => columns,
                _ => 2,
            };
            (
                group.layout.direction,
                cols,
                group.layout.spacing,
                group.layout.padding,
                group.layout.alignment.horizontal,
                group.layout.alignment.vertical,
                group.layout.item_width,
                group.layout.item_height,
                group.children.len(),
                group.nav_up,
                group.nav_down,
                group.nav_left,
                group.nav_right,
            )
        };

        // Direction dropdown
        ctx.draw_text("Direction:", x, *y + 16.0, 12.0, Color::WHITE);
        let dir_options = ["Vertical", "Horizontal", "Grid"];
        let current_dir = match direction {
            LayoutDirection::Vertical => "Vertical",
            LayoutDirection::Horizontal => "Horizontal",
            LayoutDirection::Grid { .. } => "Grid",
        };
        let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
        if let Some(selected) = Dropdown::new(
            self.properties_panel.widget_ids.layout_direction_id,
            dropdown_rect,
            current_dir,
            &dir_options,
            |s| s.to_string(),
        )
        .blocked(blocked)
        .show(ctx)
        {
            let new_dir = match selected {
                "Vertical" => LayoutDirection::Vertical,
                "Horizontal" => LayoutDirection::Horizontal,
                "Grid" => LayoutDirection::Grid { columns: grid_cols },
                _ => direction,
            };
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    group.layout.direction = new_dir;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Grid columns (only if Grid)
        if matches!(direction, LayoutDirection::Grid { .. }) {
            ctx.draw_text("Columns:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
            let new_cols = NumberInput::new(
                self.properties_panel.widget_ids.layout_grid_cols_id,
                field_rect,
                grid_cols as f32,
            )
            .blocked(blocked)
            .min(1.0)
            .max(20.0)
            .show(ctx);
            let new_cols = new_cols as u32;
            if new_cols != grid_cols {
                if let Some(element) = self.selected_element_mut() {
                    if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                        group.layout.direction = LayoutDirection::Grid { columns: new_cols };
                    }
                }
            }
            *y += ROW_HEIGHT;
        }

        // Spacing
        ctx.draw_text("Spacing:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_spacing = NumberInput::new(
            self.properties_panel.widget_ids.layout_spacing_id,
            field_rect,
            spacing,
        )
        .blocked(blocked)
        .min(0.0)
        .show(ctx);
        if (new_spacing - spacing).abs() > 0.01 {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    group.layout.spacing = new_spacing;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Padding
        *y += 4.0;
        ctx.draw_text("Padding", x, *y + 14.0, 12.0, Color::GREY);
        *y += 20.0;

        let pad_fields = [
            ("Top:", self.properties_panel.widget_ids.layout_pad_top_id, padding.top),
            ("Right:", self.properties_panel.widget_ids.layout_pad_right_id, padding.right),
            ("Bottom:", self.properties_panel.widget_ids.layout_pad_bottom_id, padding.bottom),
            ("Left:", self.properties_panel.widget_ids.layout_pad_left_id, padding.left),
        ];

        for (label, id, current_val) in pad_fields {
            ctx.draw_text(label, x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
            let new_val = NumberInput::new(id, field_rect, current_val)
                .blocked(blocked)
                .min(0.0)
                .show(ctx);
            if (new_val - current_val).abs() > 0.01 {
                if let Some(element) = self.selected_element_mut() {
                    if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                        match label {
                            "Top:" => group.layout.padding.top = new_val,
                            "Right:" => group.layout.padding.right = new_val,
                            "Bottom:" => group.layout.padding.bottom = new_val,
                            "Left:" => group.layout.padding.left = new_val,
                            _ => {}
                        }
                    }
                }
            }
            *y += ROW_HEIGHT;
        }

        // Alignment
        *y += 4.0;
        ctx.draw_text("Alignment", x, *y + 14.0, 12.0, Color::GREY);
        *y += 20.0;

        // Horizontal alignment
        ctx.draw_text("H Align:", x, *y + 16.0, 12.0, Color::WHITE);
        let h_options = ["Left", "Center", "Right"];
        let current_h = match h_align {
            HorizontalAlign::Left => "Left",
            HorizontalAlign::Center => "Center",
            HorizontalAlign::Right => "Right",
        };
        let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
        if let Some(selected) = Dropdown::new(
            self.properties_panel.widget_ids.layout_h_align_id,
            dropdown_rect,
            current_h,
            &h_options,
            |s| s.to_string(),
        )
        .blocked(blocked)
        .show(ctx)
        {
            let new_align = match selected {
                "Left" => HorizontalAlign::Left,
                "Center" => HorizontalAlign::Center,
                "Right" => HorizontalAlign::Right,
                _ => h_align,
            };
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    group.layout.alignment.horizontal = new_align;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Vertical alignment
        ctx.draw_text("V Align:", x, *y + 16.0, 12.0, Color::WHITE);
        let v_options = ["Top", "Middle", "Bottom"];
        let current_v = match v_align {
            VerticalAlign::Top => "Top",
            VerticalAlign::Middle => "Middle",
            VerticalAlign::Bottom => "Bottom",
        };
        let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
        if let Some(selected) = Dropdown::new(
            self.properties_panel.widget_ids.layout_v_align_id,
            dropdown_rect,
            current_v,
            &v_options,
            |s| s.to_string(),
        )
        .blocked(blocked)
        .show(ctx)
        {
            let new_align = match selected {
                "Top" => VerticalAlign::Top,
                "Middle" => VerticalAlign::Middle,
                "Bottom" => VerticalAlign::Bottom,
                _ => v_align,
            };
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    group.layout.alignment.vertical = new_align;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Item size
        *y += 4.0;
        ctx.draw_text("Item Size", x, *y + 14.0, 12.0, Color::GREY);
        *y += 20.0;

        ctx.draw_text("Width:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_item_w = NumberInput::new(
            self.properties_panel.widget_ids.layout_item_w_id,
            field_rect,
            item_w,
        )
        .blocked(blocked)
        .min(1.0)
        .show(ctx);
        if (new_item_w - item_w).abs() > 0.01 {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    group.layout.item_width = new_item_w;
                }
            }
        }
        *y += ROW_HEIGHT;

        ctx.draw_text("Height:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_item_h = NumberInput::new(
            self.properties_panel.widget_ids.layout_item_h_id,
            field_rect,
            item_h,
        )
        .blocked(blocked)
        .min(1.0)
        .show(ctx);
        if (new_item_h - item_h).abs() > 0.01 {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    group.layout.item_height = new_item_h;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Children list
        *y += 4.0;
        ctx.draw_text(
            &format!("Children ({})", child_count),
            x,
            *y + 14.0,
            12.0,
            Color::GREY,
        );
        *y += 20.0;

        // Managed toggle
        for i in 0..child_count {
            let (child_label, managed) = {
                let Some(element) = self.selected_element() else { break };
                let MenuElementKind::LayoutGroup(group) = &element.kind else { break };
                let child = &group.children[i];
                let label = if !child.element.name.is_empty() {
                    child.element.name.clone()
                } else {
                    match &child.element.kind {
                        MenuElementKind::Label(l) => format!("Label: {}", l.text_key),
                        MenuElementKind::Button(b) => format!("Button: {}", b.text_key),
                        MenuElementKind::Panel(_) => "Panel".to_string(),
                        MenuElementKind::LayoutGroup(_) => "Layout Group".to_string(),
                    }
                };
                (label, child.managed)
            };

            ctx.draw_text(&child_label, x + 20.0, *y + 16.0, 11.0, Color::WHITE);

            let checkbox_rect = Rect::new(x, *y + 4.0, 16.0, 16.0);
            let mut managed_val = managed;
            if gui_checkbox(ctx, checkbox_rect, &mut managed_val) {
                if let Some(element) = self.selected_element_mut() {
                    if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                        if let Some(child) = group.children.get_mut(i) {
                            child.managed = managed_val;
                        }
                    }
                }
            }
            *y += ROW_HEIGHT;
        }

        // Navigation section
        *y += 8.0;
        ctx.draw_text("Navigation", x, *y + 14.0, 12.0, Color::GREY);
        *y += 20.0;

        let focusable_elements = self.get_focusable_element_names();
        self.draw_layout_nav_dropdown(ctx, y, x, w, "Nav Up:", self.properties_panel.widget_ids.layout_nav_up_id, nav_up, &focusable_elements, blocked, |group, idx| group.nav_up = idx);
        self.draw_layout_nav_dropdown(ctx, y, x, w, "Nav Down:", self.properties_panel.widget_ids.layout_nav_down_id, nav_down, &focusable_elements, blocked, |group, idx| group.nav_down = idx);
        self.draw_layout_nav_dropdown(ctx, y, x, w, "Nav Left:", self.properties_panel.widget_ids.layout_nav_left_id, nav_left, &focusable_elements, blocked, |group, idx| group.nav_left = idx);
        self.draw_layout_nav_dropdown(ctx, y, x, w, "Nav Right:", self.properties_panel.widget_ids.layout_nav_right_id, nav_right, &focusable_elements, blocked, |group, idx| group.nav_right = idx);
    }

    fn draw_layout_nav_dropdown<F>(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        label: &str,
        id: WidgetId,
        current: Option<usize>,
        options: &[(usize, String)],
        blocked: bool,
        mut setter: F,
    ) where
        F: FnMut(&mut LayoutGroupElement, Option<usize>),
    {
        ctx.draw_text(label, x, *y + 16.0, 12.0, Color::WHITE);

        let current_label = current
            .and_then(|idx| options.iter().find(|(i, _)| *i == idx))
            .map(|(_, name)| name.as_str())
            .unwrap_or("None");

        let mut nav_options: Vec<String> = vec!["None".to_string()];
        nav_options.extend(options.iter().map(|(_, name)| name.clone()));

        let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
        if let Some(selected) = Dropdown::new(
            id,
            dropdown_rect,
            current_label,
            &nav_options,
            |s| s.clone(),
        )
        .blocked(blocked)
        .show(ctx)
        {
            let new_nav = if selected == "None" {
                None
            } else {
                options.iter().find(|(_, name)| name == &selected).map(|(idx, _)| *idx)
            };

            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                    setter(group, new_nav);
                }
            }
        }
        *y += ROW_HEIGHT;
    }
}
