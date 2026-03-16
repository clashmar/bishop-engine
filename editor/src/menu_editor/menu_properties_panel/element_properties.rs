// editor/src/menu_editor/menu_properties_panel/element_properties.rs
use crate::menu_editor::MenuEditor;
use super::{ROW_HEIGHT, LABEL_WIDTH, FIELD_HEIGHT, common_properties::row_visible};
use engine_core::prelude::*;
use bishop::prelude::*;

impl MenuEditor {
    pub(super) fn draw_label_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
        clip: &Rect,
    ) {
        let (current_text_key, current_font_size, current_alignment) = {
            let Some(element) = self.selected_element() else { return };
            let MenuElementKind::Label(label) = &element.kind else { return };
            (label.text_key.clone(), label.font_size, label.alignment)
        };

        // Text key field
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Text Key:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);

            let (new_text_key, _) = TextInput::new(
                self.properties_panel.widget_ids.text_id,
                field_rect,
                &current_text_key
            )
            .blocked(blocked)
            .show(ctx);

            if new_text_key != current_text_key {
                self.push_element_update(|el| {
                    if let MenuElementKind::Label(label) = &mut el.kind {
                        label.text_key = new_text_key;
                    }
                });
            }
        }
        *y += ROW_HEIGHT;

        // Font size
        if row_visible(*y, ROW_HEIGHT, clip) {
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
                self.push_element_update(|el| {
                    if let MenuElementKind::Label(label) = &mut el.kind {
                        label.font_size = new_font_size;
                    }
                });
            }
        }
        *y += ROW_HEIGHT;

        // Horizontal alignment
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Align:", x, *y + 16.0, 12.0, Color::WHITE);
            let h_options = ["Left", "Center", "Right"];
            let current_h = match current_alignment {
                HorizontalAlign::Left => "Left",
                HorizontalAlign::Center => "Center",
                HorizontalAlign::Right => "Right",
            };
            let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            if let Some(selected) = Dropdown::new(
                self.properties_panel.widget_ids.label_h_align_id,
                dropdown_rect,
                current_h,
                &h_options,
                |s| s.to_string(),
            )
            .blocked(blocked)
            .fixed_width()
            .show(ctx)
            {
                let new_align = match selected {
                    "Left" => HorizontalAlign::Left,
                    "Center" => HorizontalAlign::Center,
                    "Right" => HorizontalAlign::Right,
                    _ => current_alignment,
                };
                self.push_element_update(|el| {
                    if let MenuElementKind::Label(label) = &mut el.kind {
                        label.alignment = new_align;
                    }
                });
            }
        }
        *y += ROW_HEIGHT;
    }

    pub(super) fn draw_button_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
        clip: &Rect,
    ) {
        let (current_text_key, current_font_size, current_action, nav_up, nav_down, nav_left, nav_right) = {
            let Some(element) = self.selected_element() else { return };
            let MenuElementKind::Button(button) = &element.kind else { return };
            (
                button.text_key.clone(),
                button.font_size,
                button.action.clone(),
                button.nav_up,
                button.nav_down,
                button.nav_left,
                button.nav_right,
            )
        };

        // Text key field
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Text Key:", x, *y + 16.0, 12.0, Color::WHITE);

            let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);

            let (new_text_key, _) = TextInput::new(
                self.properties_panel.widget_ids.text_id,
                field_rect,
                &current_text_key
            )
            .blocked(blocked)
            .show(ctx);

            if new_text_key != current_text_key {
                self.push_element_update(|el| {
                    if let MenuElementKind::Button(button) = &mut el.kind {
                        button.text_key = new_text_key;
                    }
                });
            }
        }
        *y += ROW_HEIGHT;

        // Font size
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Font Size:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, 60.0, FIELD_HEIGHT);
            let new_font_size = NumberInput::new(self.properties_panel.widget_ids.font_size_id, field_rect, current_font_size)
            .blocked(blocked)
            .min(8.0)
            .max(72.0)
            .show(ctx);

            if (new_font_size - current_font_size).abs() > 0.01 {
                self.push_element_update(|el| {
                    if let MenuElementKind::Button(button) = &mut el.kind {
                        button.font_size = new_font_size;
                    }
                });
            }
        }
        *y += ROW_HEIGHT;

        // Action dropdown
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Action:", x, *y + 16.0, 12.0, Color::WHITE);
            let action_variants = [
                MenuAction::Resume,
                MenuAction::CloseMenu,
                MenuAction::QuitToMainMenu,
                MenuAction::QuitGame,
                MenuAction::OpenMenu(String::new()),
                MenuAction::Custom(String::new()),
            ];
            let action_options: Vec<&str> = action_variants.iter().map(|a| a.ui_label()).collect();
            let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            if let Some(selected) = Dropdown::new(
                self.properties_panel.widget_ids.action_id,
                dropdown_rect,
                current_action.ui_label(),
                &action_options,
                |s| s.to_string(),
            )
            .blocked(blocked)
            .fixed_width()
            .show(ctx)
            {
                if let Some(new_action) = action_variants.into_iter()
                    .find(|a| a.ui_label() == selected)
                {
                    self.push_element_update(|el| {
                        if let MenuElementKind::Button(button) = &mut el.kind {
                            button.action = new_action;
                        }
                    });
                }
            }
        }
        *y += ROW_HEIGHT;

        // Action parameter (for OpenMenu/Custom)
        let needs_param = matches!(current_action, MenuAction::OpenMenu(_) | MenuAction::Custom(_));
        if needs_param {
            if row_visible(*y, ROW_HEIGHT, clip) {
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
                    self.push_element_update(|el| {
                        if let MenuElementKind::Button(button) = &mut el.kind {
                            button.action = match &button.action {
                                MenuAction::OpenMenu(_) => MenuAction::OpenMenu(new_param),
                                MenuAction::Custom(_) => MenuAction::Custom(new_param),
                                other => other.clone(),
                            };
                        }
                    });
                }
            }
            *y += ROW_HEIGHT;
        }

        // Navigation section (only for top-level buttons, not children of layout groups)
        if self.selected_child_index.is_none() {
            *y += 8.0;
            if row_visible(*y, 20.0, clip) {
                ctx.draw_text("Navigation", x, *y + 14.0, 12.0, Color::GREY);
            }
            *y += 20.0;

            let focusable_elements = self.get_focusable_element_names();
            self.draw_nav_dropdown(ctx, y, x, w, "Nav Up:", self.properties_panel.widget_ids.nav_up_id, nav_up, &focusable_elements, blocked, clip, |btn, idx| btn.nav_up = idx);
            self.draw_nav_dropdown(ctx, y, x, w, "Nav Down:", self.properties_panel.widget_ids.nav_down_id, nav_down, &focusable_elements, blocked, clip, |btn, idx| btn.nav_down = idx);
            self.draw_nav_dropdown(ctx, y, x, w, "Nav Left:", self.properties_panel.widget_ids.nav_left_id, nav_left, &focusable_elements, blocked, clip, |btn, idx| btn.nav_left = idx);
            self.draw_nav_dropdown(ctx, y, x, w, "Nav Right:", self.properties_panel.widget_ids.nav_right_id, nav_right, &focusable_elements, blocked, clip, |btn, idx| btn.nav_right = idx);
        }
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
        clip: &Rect,
        mut setter: F,
    ) where
        F: FnMut(&mut ButtonElement, Option<usize>),
    {
        if row_visible(*y, ROW_HEIGHT, clip) {
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
            .fixed_width()
            .show(ctx)
            {
                let new_nav = if selected == "None" {
                    None
                } else {
                    options.iter().find(|(_, name)| name == &selected).map(|(idx, _)| *idx)
                };

                self.push_element_update(|el| {
                    if let MenuElementKind::Button(button) = &mut el.kind {
                        setter(button, new_nav);
                    }
                });
            }
        }
        *y += ROW_HEIGHT;
    }

    pub(super) fn get_focusable_element_names(&self) -> Vec<(usize, String)> {
        let Some(template) = self.current_template() else {
            return Vec::new();
        };

        let selected = self.primary_selected_index();
        template
            .elements
            .iter()
            .enumerate()
            .filter(|(idx, _)| selected != Some(*idx))
            .filter_map(|(idx, element)| {
                let name = if !element.name.is_empty() {
                    element.name.clone()
                } else {
                    match &element.kind {
                        MenuElementKind::Button(button) => button.text_key.clone(),
                        MenuElementKind::LayoutGroup(group) => {
                            let button_count = group.children.iter()
                                .filter(|c| matches!(c.element.kind, MenuElementKind::Button(_)))
                                .count();
                            format!("Layout Group ({} buttons)", button_count)
                        }
                        _ => return None,
                    }
                };
                match &element.kind {
                    MenuElementKind::Button(_) | MenuElementKind::LayoutGroup(_) => {
                        Some((idx, format!("{}: {}", idx, name)))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    pub(super) fn draw_panel_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
        clip: &Rect,
    ) {
        let (current_color, current_opacity) = {
            let Some(element) = self.selected_element() else { return };
            let MenuElementKind::Panel(panel) = &element.kind else { return };
            let color = match panel.background.fill {
                PanelFill::SolidColor(c) => c,
            };
            (color, panel.background.opacity)
        };

        if row_visible(*y, 20.0, clip) {
            ctx.draw_text("Background", x, *y + 14.0, 12.0, Color::GREY);
        }
        *y += 20.0;

        // Color
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Color:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            let new_color = ColorInput::new(
                self.properties_panel.widget_ids.panel_color_id,
                field_rect,
                current_color,
            )
            .blocked(blocked)
            .show(ctx);
            if new_color != current_color {
                self.push_element_update(|el| {
                    if let MenuElementKind::Panel(panel) = &mut el.kind {
                        panel.background.fill = PanelFill::SolidColor(new_color);
                    }
                });
            }
        }
        *y += ROW_HEIGHT;

        // Opacity
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Opacity:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            let (new_opacity, changed) = gui_slider(
                ctx,
                self.properties_panel.widget_ids.panel_opacity_id,
                field_rect,
                0.0,
                1.0,
                current_opacity,
            );
            if changed {
                self.push_element_update(|el| {
                    if let MenuElementKind::Panel(panel) = &mut el.kind {
                        panel.background.opacity = new_opacity;
                    }
                });
            }
        }
        *y += ROW_HEIGHT;
    }
}
