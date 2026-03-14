// editor/src/menu_editor/menu_properties_panel.rs
use crate::menu_editor::MenuEditor;
use engine_core::prelude::*;
use bishop::prelude::*;

const ROW_HEIGHT: f32 = 28.0;
const LABEL_WIDTH: f32 = 80.0;
const FIELD_HEIGHT: f32 = 24.0;

/// Widget IDs for the properties module.
#[derive(Default)]
pub struct PropertiesWidgetIds {
    text_id: WidgetId,
    font_size_id: WidgetId,
    action_id: WidgetId,
    action_param_id: WidgetId,
    z_order_id: WidgetId,
    pos_x_id: WidgetId,
    pos_y_id: WidgetId,
    size_w_id: WidgetId,
    size_h_id: WidgetId,
    nav_up_id: WidgetId,
    nav_down_id: WidgetId,
    nav_left_id: WidgetId,
    nav_right_id: WidgetId,
    layout_direction_id: WidgetId,
    layout_grid_cols_id: WidgetId,
    layout_spacing_id: WidgetId,
    layout_pad_top_id: WidgetId,
    layout_pad_right_id: WidgetId,
    layout_pad_bottom_id: WidgetId,
    layout_pad_left_id: WidgetId,
    layout_h_align_id: WidgetId,
    layout_v_align_id: WidgetId,
    layout_item_w_id: WidgetId,
    layout_item_h_id: WidgetId,
}

/// Groups property panel data.
pub struct MenuPropertiesPanel {
    scroll_y: f32,
    widget_ids: PropertiesWidgetIds,
}

impl MenuPropertiesPanel {
    /// Creates a new properties panel.
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            widget_ids: PropertiesWidgetIds::default(),
        }
    }
}

impl MenuEditor {
    /// Renders the properties panel and handles editing.
    pub fn draw_properties_panel(
        &mut self, 
        ctx: &mut WgpuContext, 
        rect: Rect, 
        blocked: bool
    ) {
        let content_height = self.calculate_properties_height();

        let properties_panel = &mut self.properties_panel;
        let mouse: Vec2 = ctx.mouse_position().into();

        if !blocked && rect.contains(mouse) {
            let (_, wheel_y) = ctx.mouse_wheel();
            properties_panel.scroll_y += wheel_y * 20.0;
        }

        let scroll_range = (content_height - rect.h).max(0.0);
        properties_panel.scroll_y = properties_panel.scroll_y.clamp(-scroll_range, 0.0);

        let mut y = rect.y + properties_panel.scroll_y + 8.0;
        let content_x = rect.x + 8.0;
        let content_w = rect.w - 16.0;

        ctx.draw_text("Properties", content_x, y + 14.0, 14.0, Color::GREY);
        y += 24.0;

        if self.selected_element_index.is_none() {
            ctx.draw_text(
                "No element selected",
                content_x,
                y + 14.0,
                12.0,
                Color::new(0.6, 0.6, 0.6, 1.0),
            );
            return;
        }

        let element_kind = self
            .selected_element()
            .map(|e| e.kind.clone());

        let Some(kind) = element_kind else { return };

        match kind {
            MenuElementKind::Label(_) => {
                self.draw_label_properties(ctx, &mut y, content_x, content_w, blocked);
            }
            MenuElementKind::Button(_) => {
                self.draw_button_properties(ctx, &mut y, content_x, content_w, blocked);
            }
            MenuElementKind::Panel(_) => {
                self.draw_panel_properties(ctx, &mut y, content_x, content_w, blocked);
            }
            MenuElementKind::LayoutGroup(_) => {
                self.draw_layout_group_properties(ctx, &mut y, content_x, content_w, blocked);
            }
        }

        y += 8.0;
        self.draw_common_properties(ctx, &mut y, content_x, content_w, blocked);
    }

    fn draw_label_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
    ) {
        let (current_text, current_font_size) = {
            let Some(element) = self.selected_element() else { return };
            let MenuElementKind::Label(label) = &element.kind else { return };
            (label.text.clone(), label.font_size)
        };

        // Text field
        ctx.draw_text("Text:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);

        let (new_text, _) = TextInput::new(
            self.properties_panel.widget_ids.text_id, 
            field_rect, 
            &current_text
        )
        .blocked(blocked)
        .show(ctx);

        if new_text != current_text {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::Label(label) = &mut element.kind {
                    label.text = new_text;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Font size
        ctx.draw_text("Font Size:", x, *y + 16.0, 12.0, Color::WHITE);

        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);

        let new_font_size = NumberInput::new(
            self.properties_panel.widget_ids.font_size_id, 
            field_rect, 
            current_font_size
        )
        .blocked(blocked)
        .min(8.0)
        .max(72.0)
        .show(ctx);

        if (new_font_size - current_font_size).abs() > 0.01 {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::Label(label) = &mut element.kind {
                    label.font_size = new_font_size;
                }
            }
        }
        *y += ROW_HEIGHT;
    }

    fn draw_button_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
    ) {
        let (current_text, current_font_size, current_action, nav_up, nav_down, nav_left, nav_right) = {
            let Some(element) = self.selected_element() else { return };
            let MenuElementKind::Button(button) = &element.kind else { return };
            (
                button.text.clone(),
                button.font_size,
                button.action.clone(),
                button.nav_up,
                button.nav_down,
                button.nav_left,
                button.nav_right,
            )
        };

        // Text field
        ctx.draw_text("Text:", x, *y + 16.0, 12.0, Color::WHITE);

        let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);

        let (new_text, _) = TextInput::new(
            self.properties_panel.widget_ids.text_id, 
            field_rect, 
            &current_text
        )
        .blocked(blocked)
        .show(ctx);

        if new_text != current_text {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::Button(button) = &mut element.kind {
                    button.text = new_text;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Font size
        ctx.draw_text("Font Size:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_font_size = NumberInput::new(self.properties_panel.widget_ids.font_size_id, field_rect, current_font_size)
        .blocked(blocked)
        .min(8.0)
        .max(72.0)
        .show(ctx);

        if (new_font_size - current_font_size).abs() > 0.01 {
            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::Button(button) = &mut element.kind {
                    button.font_size = new_font_size;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Action dropdown
        ctx.draw_text("Action:", x, *y + 16.0, 12.0, Color::WHITE);
        let action_options = [
            "Resume",
            "CloseMenu",
            "QuitToMainMenu",
            "QuitGame",
            "OpenMenu",
            "Custom",
        ];
        let current_action_str = match &current_action {
            MenuAction::Resume => "Resume",
            MenuAction::CloseMenu => "CloseMenu",
            MenuAction::QuitToMainMenu => "QuitToMainMenu",
            MenuAction::QuitGame => "QuitGame",
            MenuAction::OpenMenu(_) => "OpenMenu",
            MenuAction::Custom(_) => "Custom",
        };
        let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
        if let Some(selected) = Dropdown::new(
            self.properties_panel.widget_ids.action_id,
            dropdown_rect,
            current_action_str,
            &action_options,
            |s| s.to_string(),
        )
        .blocked(blocked)
        .show(ctx)
        {
            let new_action = match selected {
                "Resume" => MenuAction::Resume,
                "CloseMenu" => MenuAction::CloseMenu,
                "QuitToMainMenu" => MenuAction::QuitToMainMenu,
                "QuitGame" => MenuAction::QuitGame,
                "OpenMenu" => MenuAction::OpenMenu(String::new()),
                "Custom" => MenuAction::Custom(String::new()),
                _ => current_action.clone(),
            };

            if let Some(element) = self.selected_element_mut() {
                if let MenuElementKind::Button(button) = &mut element.kind {
                    button.action = new_action;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Action parameter (for OpenMenu/Custom)
        let needs_param = matches!(current_action, MenuAction::OpenMenu(_) | MenuAction::Custom(_));
        if needs_param {
            let param_value = match &current_action {
                MenuAction::OpenMenu(s) | MenuAction::Custom(s) => s.clone(),
                _ => String::new(),
            };

            ctx.draw_text("Param:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            let (new_param, _) = TextInput::new(self.properties_panel.widget_ids.action_param_id, field_rect, &param_value)
                .blocked(blocked)
                .show(ctx);

            if new_param != param_value {
                if let Some(element) = self.selected_element_mut() {
                    if let MenuElementKind::Button(button) = &mut element.kind {
                        button.action = match &button.action {
                            MenuAction::OpenMenu(_) => MenuAction::OpenMenu(new_param),
                            MenuAction::Custom(_) => MenuAction::Custom(new_param),
                            other => other.clone(),
                        };
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
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Up:", self.properties_panel.widget_ids.nav_up_id, nav_up, &focusable_elements, blocked, |btn, idx| btn.nav_up = idx);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Down:", self.properties_panel.widget_ids.nav_down_id, nav_down, &focusable_elements, blocked, |btn, idx| btn.nav_down = idx);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Left:", self.properties_panel.widget_ids.nav_left_id, nav_left, &focusable_elements, blocked, |btn, idx| btn.nav_left = idx);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Right:", self.properties_panel.widget_ids.nav_right_id, nav_right, &focusable_elements, blocked, |btn, idx| btn.nav_right = idx);
    }

    fn draw_nav_dropdown<F>(
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
        F: FnMut(&mut ButtonElement, Option<usize>),
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
                if let MenuElementKind::Button(button) = &mut element.kind {
                    setter(button, new_nav);
                }
            }
        }
        *y += ROW_HEIGHT;
    }

    fn get_focusable_element_names(&self) -> Vec<(usize, String)> {
        let Some(template) = self.current_template() else {
            return Vec::new();
        };

        template
            .elements
            .iter()
            .enumerate()
            .filter_map(|(idx, element)| {
                if let MenuElementKind::Button(button) = &element.kind {
                    Some((idx, format!("{}: {}", idx, button.text)))
                } else {
                    None
                }
            })
            .collect()
    }

    fn draw_panel_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        _w: f32,
        _blocked: bool,
    ) {
        ctx.draw_text("Type:", x, *y + 16.0, 12.0, Color::WHITE);
        ctx.draw_text("Panel", x + LABEL_WIDTH, *y + 16.0, 12.0, Color::new(0.7, 0.7, 0.7, 1.0));
        *y += ROW_HEIGHT;
    }

    fn draw_layout_group_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
    ) {
        let (direction, grid_cols, spacing, padding, h_align, v_align, item_w, item_h, child_count) = {
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
            )
        };

        ctx.draw_text("Type:", x, *y + 16.0, 12.0, Color::WHITE);
        ctx.draw_text("Layout Group", x + LABEL_WIDTH, *y + 16.0, 12.0, Color::new(0.7, 0.7, 0.7, 1.0));
        *y += ROW_HEIGHT;

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

        // Show children with managed toggle
        for i in 0..child_count {
            let (child_label, managed) = {
                let Some(element) = self.selected_element() else { break };
                let MenuElementKind::LayoutGroup(group) = &element.kind else { break };
                let child = &group.children[i];
                let label = match &child.element.kind {
                    MenuElementKind::Label(l) => format!("Label: {}", l.text),
                    MenuElementKind::Button(b) => format!("Button: {}", b.text),
                    MenuElementKind::Panel(_) => "Panel".to_string(),
                    MenuElementKind::LayoutGroup(_) => "Layout Group".to_string(),
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
    }

    fn draw_common_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        _w: f32,
        blocked: bool,
    ) {
        let (rect_val, enabled, visible, z_order) = {
            let Some(element) = self.selected_element() else { return };
            (element.rect, element.enabled, element.visible, element.z_order)
        };
        let child_is_managed = self.is_selected_child_managed();

        // Z Order
        ctx.draw_text("Z Order:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_z = NumberInput::new(self.properties_panel.widget_ids.z_order_id, field_rect, z_order as f32)
            .blocked(blocked)
            .show(ctx);
        let new_z = new_z as i32;
        if new_z != z_order {
            if let Some(element) = self.selected_element_mut() {
                element.z_order = new_z;
            }
        }
        *y += ROW_HEIGHT;

        if !child_is_managed {
            ctx.draw_text("Position (normalized)", x, *y + 14.0, 12.0, Color::GREY);
            *y += 20.0;

            // Position X
            ctx.draw_text("X:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + 24.0, *y, 60.0, FIELD_HEIGHT);
            let new_x = NumberInput::new(self.properties_panel.widget_ids.pos_x_id, field_rect, rect_val.x)
                .blocked(blocked)
                .show(ctx);
            let px_x = format!("{}px", (new_x * DESIGN_RESOLUTION_WIDTH) as i32);
            ctx.draw_text(&px_x, x + 88.0, *y + 16.0, 10.0, Color::GREY);

            // Position Y
            ctx.draw_text("Y:", x + 130.0, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + 154.0, *y, 60.0, FIELD_HEIGHT);
            let new_y = NumberInput::new(self.properties_panel.widget_ids.pos_y_id, field_rect, rect_val.y)
                .blocked(blocked)
                .show(ctx);

            if (new_x - rect_val.x).abs() > 0.001 || (new_y - rect_val.y).abs() > 0.001 {
                if let Some(element) = self.selected_element_mut() {
                    element.rect.x = new_x;
                    element.rect.y = new_y;
                }
            }
            *y += ROW_HEIGHT;

            // Size W
            ctx.draw_text("W:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + 24.0, *y, 60.0, FIELD_HEIGHT);
            let new_w = NumberInput::new(self.properties_panel.widget_ids.size_w_id, field_rect, rect_val.w)
                .blocked(blocked)
                .min(0.005)
                .show(ctx);
            let px_w = format!("{}px", (new_w * DESIGN_RESOLUTION_WIDTH) as i32);
            ctx.draw_text(&px_w, x + 88.0, *y + 16.0, 10.0, Color::GREY);

            // Size H
            ctx.draw_text("H:", x + 130.0, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + 154.0, *y, 60.0, FIELD_HEIGHT);
            let new_h = NumberInput::new(self.properties_panel.widget_ids.size_h_id, field_rect, rect_val.h)
                .blocked(blocked)
                .min(0.005)
                .show(ctx);

            if (new_w - rect_val.w).abs() > 0.001 || (new_h - rect_val.h).abs() > 0.001 {
                if let Some(element) = self.selected_element_mut() {
                    element.rect.w = new_w;
                    element.rect.h = new_h;
                }
            }
            *y += ROW_HEIGHT + 8.0;
        } else {
            ctx.draw_text(
                "Position/size managed by layout",
                x,
                *y + 14.0,
                10.0,
                Color::new(0.5, 0.5, 0.5, 1.0),
            );
            *y += 20.0;
        }

        // Enabled checkbox
        ctx.draw_text("Enabled:", x, *y + 16.0, 12.0, Color::WHITE);
        let checkbox_rect = Rect::new(x + LABEL_WIDTH, *y + 4.0, 16.0, 16.0);
        let mut enabled_val = enabled;
        if gui_checkbox(ctx, checkbox_rect, &mut enabled_val) {
            if let Some(element) = self.selected_element_mut() {
                element.enabled = enabled_val;
            }
        }
        *y += ROW_HEIGHT;

        // Visible checkbox
        ctx.draw_text("Visible:", x, *y + 16.0, 12.0, Color::WHITE);
        let checkbox_rect = Rect::new(x + LABEL_WIDTH, *y + 4.0, 16.0, 16.0);
        let mut visible_val = visible;
        if gui_checkbox(ctx, checkbox_rect, &mut visible_val) {
            if let Some(element) = self.selected_element_mut() {
                element.visible = visible_val;
            }
        }
        *y += ROW_HEIGHT;
    }

    fn calculate_properties_height(&self) -> f32 {
        let base_height = 200.0;

        let element_height = match self.selected_element().map(|e| &e.kind) {
            Some(MenuElementKind::Label(_)) => ROW_HEIGHT * 2.0,
            Some(MenuElementKind::Button(btn)) => {
                let param_row = if matches!(btn.action, MenuAction::OpenMenu(_) | MenuAction::Custom(_)) {
                    ROW_HEIGHT
                } else {
                    0.0
                };
                ROW_HEIGHT * 3.0 + param_row + 28.0 + ROW_HEIGHT * 4.0
            }
            Some(MenuElementKind::Panel(_)) => ROW_HEIGHT,
            Some(MenuElementKind::LayoutGroup(group)) => {
                let grid_row = if matches!(group.layout.direction, LayoutDirection::Grid { .. }) {
                    ROW_HEIGHT
                } else {
                    0.0
                };
                ROW_HEIGHT * (1.0 + 1.0 + 1.0 + 4.0 + 2.0 + 2.0)
                    + grid_row
                    + 20.0 * 3.0 // section headers
                    + 4.0 * 3.0  // section gaps
                    + 20.0 // children header
                    + ROW_HEIGHT * group.children.len() as f32
            }
            None => 0.0,
        };

        // Managed children skip position and size rows (2 rows + 20px header + 8px gap)
        let pos_size_height = if self.is_selected_child_managed() {
            20.0 // just the "managed" notice line
        } else {
            ROW_HEIGHT * 2.0 + 20.0 + 8.0
        };

        let common_height = ROW_HEIGHT * 3.0 + pos_size_height + 8.0;

        base_height + element_height + common_height
    }
}

impl Default for MenuPropertiesPanel {
    fn default() -> Self {
        Self::new()
    }
}
