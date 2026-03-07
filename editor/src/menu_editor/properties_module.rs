// editor/src/menu_editor/properties_module.rs
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
    spacer_size_id: WidgetId,
    pos_x_id: WidgetId,
    pos_y_id: WidgetId,
    size_w_id: WidgetId,
    size_h_id: WidgetId,
    nav_up_id: WidgetId,
    nav_down_id: WidgetId,
    nav_left_id: WidgetId,
    nav_right_id: WidgetId,
}

/// Properties editor for the selected menu element.
pub struct PropertiesModule {
    scroll_y: f32,
    widget_ids: PropertiesWidgetIds,
}

impl PropertiesModule {
    /// Creates a new properties module.
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            widget_ids: PropertiesWidgetIds::default(),
        }
    }

    /// Renders the properties panel and handles editing.
    pub fn draw(&mut self, ctx: &mut WgpuContext, rect: Rect, menu_editor: &mut MenuEditor, blocked: bool) {
        let mouse: Vec2 = ctx.mouse_position().into();

        if !blocked && rect.contains(mouse) {
            let (_, wheel_y) = ctx.mouse_wheel();
            self.scroll_y += wheel_y * 20.0;
        }

        let content_height = self.calculate_content_height(menu_editor);
        let scroll_range = (content_height - rect.h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        let mut y = rect.y + self.scroll_y + 8.0;
        let content_x = rect.x + 8.0;
        let content_w = rect.w - 16.0;

        ctx.draw_text("Properties", content_x, y + 14.0, 14.0, Color::GREY);
        y += 24.0;

        if menu_editor.selected_element_index.is_none() {
            ctx.draw_text(
                "No element selected",
                content_x,
                y + 14.0,
                12.0,
                Color::new(0.6, 0.6, 0.6, 1.0),
            );
            return;
        }

        let element_kind = menu_editor
            .selected_element()
            .map(|e| e.kind.clone());

        let Some(kind) = element_kind else { return };

        match kind {
            MenuElementKind::Label(_) => {
                self.draw_label_properties(ctx, &mut y, content_x, content_w, menu_editor, blocked);
            }
            MenuElementKind::Button(_) => {
                self.draw_button_properties(ctx, &mut y, content_x, content_w, menu_editor, blocked);
            }
            MenuElementKind::Spacer(_) => {
                self.draw_spacer_properties(ctx, &mut y, content_x, content_w, menu_editor, blocked);
            }
            MenuElementKind::Panel(_) => {
                self.draw_panel_properties(ctx, &mut y, content_x, content_w, menu_editor, blocked);
            }
        }

        y += 8.0;
        self.draw_common_properties(ctx, &mut y, content_x, content_w, menu_editor, blocked);
    }

    fn draw_label_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        menu_editor: &mut MenuEditor,
        blocked: bool,
    ) {
        let (current_text, current_font_size) = {
            let Some(element) = menu_editor.selected_element() else { return };
            let MenuElementKind::Label(label) = &element.kind else { return };
            (label.text.clone(), label.font_size)
        };

        // Text field
        ctx.draw_text("Text:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
        let (new_text, _) = TextInput::new(self.widget_ids.text_id, field_rect, &current_text)
            .blocked(blocked)
            .show(ctx);

        if new_text != current_text {
            if let Some(element) = menu_editor.selected_element_mut() {
                if let MenuElementKind::Label(label) = &mut element.kind {
                    label.text = new_text;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Font size
        ctx.draw_text("Font Size:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_font_size = NumberInput::new(self.widget_ids.font_size_id, field_rect, current_font_size)
            .blocked(blocked)
            .min(8.0)
            .max(72.0)
            .show(ctx);

        if (new_font_size - current_font_size).abs() > 0.01 {
            if let Some(element) = menu_editor.selected_element_mut() {
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
        menu_editor: &mut MenuEditor,
        blocked: bool,
    ) {
        let (current_text, current_font_size, current_action, nav_up, nav_down, nav_left, nav_right) = {
            let Some(element) = menu_editor.selected_element() else { return };
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
        let (new_text, _) = TextInput::new(self.widget_ids.text_id, field_rect, &current_text)
            .blocked(blocked)
            .show(ctx);

        if new_text != current_text {
            if let Some(element) = menu_editor.selected_element_mut() {
                if let MenuElementKind::Button(button) = &mut element.kind {
                    button.text = new_text;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Font size
        ctx.draw_text("Font Size:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_font_size = NumberInput::new(self.widget_ids.font_size_id, field_rect, current_font_size)
            .blocked(blocked)
            .min(8.0)
            .max(72.0)
            .show(ctx);

        if (new_font_size - current_font_size).abs() > 0.01 {
            if let Some(element) = menu_editor.selected_element_mut() {
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
            self.widget_ids.action_id,
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

            if let Some(element) = menu_editor.selected_element_mut() {
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
            let (new_param, _) = TextInput::new(self.widget_ids.action_param_id, field_rect, &param_value)
                .blocked(blocked)
                .show(ctx);

            if new_param != param_value {
                if let Some(element) = menu_editor.selected_element_mut() {
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

        let focusable_elements = self.get_focusable_element_names(menu_editor);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Up:", self.widget_ids.nav_up_id, nav_up, &focusable_elements, menu_editor, blocked, |btn, idx| btn.nav_up = idx);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Down:", self.widget_ids.nav_down_id, nav_down, &focusable_elements, menu_editor, blocked, |btn, idx| btn.nav_down = idx);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Left:", self.widget_ids.nav_left_id, nav_left, &focusable_elements, menu_editor, blocked, |btn, idx| btn.nav_left = idx);
        self.draw_nav_dropdown(ctx, y, x, w, "Nav Right:", self.widget_ids.nav_right_id, nav_right, &focusable_elements, menu_editor, blocked, |btn, idx| btn.nav_right = idx);
    }

    fn draw_nav_dropdown<F>(
        &self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        label: &str,
        id: WidgetId,
        current: Option<usize>,
        options: &[(usize, String)],
        menu_editor: &mut MenuEditor,
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

            if let Some(element) = menu_editor.selected_element_mut() {
                if let MenuElementKind::Button(button) = &mut element.kind {
                    setter(button, new_nav);
                }
            }
        }
        *y += ROW_HEIGHT;
    }

    fn get_focusable_element_names(&self, menu_editor: &MenuEditor) -> Vec<(usize, String)> {
        let Some(template) = menu_editor.current_template() else {
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

    fn draw_spacer_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        _w: f32,
        menu_editor: &mut MenuEditor,
        blocked: bool,
    ) {
        let current_size = {
            let Some(element) = menu_editor.selected_element() else { return };
            let MenuElementKind::Spacer(spacer) = &element.kind else { return };
            spacer.size
        };

        ctx.draw_text("Size:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
        let new_size = NumberInput::new(self.widget_ids.spacer_size_id, field_rect, current_size)
            .blocked(blocked)
            .min(1.0)
            .max(500.0)
            .show(ctx);

        if (new_size - current_size).abs() > 0.01 {
            if let Some(element) = menu_editor.selected_element_mut() {
                if let MenuElementKind::Spacer(spacer) = &mut element.kind {
                    spacer.size = new_size;
                }
            }
        }
        *y += ROW_HEIGHT;
    }

    fn draw_panel_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        _w: f32,
        _menu_editor: &mut MenuEditor,
        _blocked: bool,
    ) {
        ctx.draw_text("Type:", x, *y + 16.0, 12.0, Color::WHITE);
        ctx.draw_text("Panel", x + LABEL_WIDTH, *y + 16.0, 12.0, Color::new(0.7, 0.7, 0.7, 1.0));
        *y += ROW_HEIGHT;
    }

    fn draw_common_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        _w: f32,
        menu_editor: &mut MenuEditor,
        blocked: bool,
    ) {
        let (rect_val, enabled, visible) = {
            let Some(element) = menu_editor.selected_element() else { return };
            (element.rect, element.enabled, element.visible)
        };

        ctx.draw_text("Position", x, *y + 14.0, 12.0, Color::GREY);
        *y += 20.0;

        // Position X
        ctx.draw_text("X:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + 24.0, *y, 60.0, FIELD_HEIGHT);
        let new_x = NumberInput::new(self.widget_ids.pos_x_id, field_rect, rect_val.x)
            .blocked(blocked)
            .show(ctx);

        // Position Y
        ctx.draw_text("Y:", x + 100.0, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + 124.0, *y, 60.0, FIELD_HEIGHT);
        let new_y = NumberInput::new(self.widget_ids.pos_y_id, field_rect, rect_val.y)
            .blocked(blocked)
            .show(ctx);

        if (new_x - rect_val.x).abs() > 0.01 || (new_y - rect_val.y).abs() > 0.01 {
            if let Some(element) = menu_editor.selected_element_mut() {
                element.rect.x = new_x;
                element.rect.y = new_y;
            }
        }
        *y += ROW_HEIGHT;

        // Size W
        ctx.draw_text("W:", x, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + 24.0, *y, 60.0, FIELD_HEIGHT);
        let new_w = NumberInput::new(self.widget_ids.size_w_id, field_rect, rect_val.w)
            .blocked(blocked)
            .min(10.0)
            .show(ctx);

        // Size H
        ctx.draw_text("H:", x + 100.0, *y + 16.0, 12.0, Color::WHITE);
        let field_rect = Rect::new(x + 124.0, *y, 60.0, FIELD_HEIGHT);
        let new_h = NumberInput::new(self.widget_ids.size_h_id, field_rect, rect_val.h)
            .blocked(blocked)
            .min(10.0)
            .show(ctx);

        if (new_w - rect_val.w).abs() > 0.01 || (new_h - rect_val.h).abs() > 0.01 {
            if let Some(element) = menu_editor.selected_element_mut() {
                element.rect.w = new_w;
                element.rect.h = new_h;
            }
        }
        *y += ROW_HEIGHT + 8.0;

        // Enabled checkbox
        ctx.draw_text("Enabled:", x, *y + 16.0, 12.0, Color::WHITE);
        let checkbox_rect = Rect::new(x + LABEL_WIDTH, *y + 4.0, 16.0, 16.0);
        let mut enabled_val = enabled;
        if gui_checkbox(ctx, checkbox_rect, &mut enabled_val) {
            if let Some(element) = menu_editor.selected_element_mut() {
                element.enabled = enabled_val;
            }
        }
        *y += ROW_HEIGHT;

        // Visible checkbox
        ctx.draw_text("Visible:", x, *y + 16.0, 12.0, Color::WHITE);
        let checkbox_rect = Rect::new(x + LABEL_WIDTH, *y + 4.0, 16.0, 16.0);
        let mut visible_val = visible;
        if gui_checkbox(ctx, checkbox_rect, &mut visible_val) {
            if let Some(element) = menu_editor.selected_element_mut() {
                element.visible = visible_val;
            }
        }
        *y += ROW_HEIGHT;
    }

    fn calculate_content_height(&self, menu_editor: &MenuEditor) -> f32 {
        let base_height = 200.0;

        let element_height = match menu_editor.selected_element().map(|e| &e.kind) {
            Some(MenuElementKind::Label(_)) => ROW_HEIGHT * 2.0,
            Some(MenuElementKind::Button(btn)) => {
                let param_row = if matches!(btn.action, MenuAction::OpenMenu(_) | MenuAction::Custom(_)) {
                    ROW_HEIGHT
                } else {
                    0.0
                };
                ROW_HEIGHT * 3.0 + param_row + 28.0 + ROW_HEIGHT * 4.0
            }
            Some(MenuElementKind::Spacer(_)) => ROW_HEIGHT,
            Some(MenuElementKind::Panel(_)) => ROW_HEIGHT,
            None => 0.0,
        };

        let common_height = ROW_HEIGHT * 5.0 + 40.0;

        base_height + element_height + common_height
    }
}

impl Default for PropertiesModule {
    fn default() -> Self {
        Self::new()
    }
}
