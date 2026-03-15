// editor/src/menu_editor/menu_properties_panel/menu_properties.rs
use crate::menu_editor::MenuEditor;
use super::{ROW_HEIGHT, LABEL_WIDTH, FIELD_HEIGHT, common_properties::row_visible};
use engine_core::prelude::*;
use bishop::prelude::*;

impl MenuEditor {
    pub(super) fn draw_menu_properties(
        &mut self,
        ctx: &mut WgpuContext,
        y: &mut f32,
        x: f32,
        w: f32,
        blocked: bool,
        clip: &Rect,
    ) {
        let Some(template) = self.current_template() else { return };
        let current_name = template.id.clone();
        let current_mode = template.mode;
        let current_bg = template.background;

        if row_visible(*y, 20.0, clip) {
            ctx.draw_text("Menu Properties", x, *y + 14.0, 12.0, Color::GREY);
        }
        *y += 20.0;

        // Name field
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Name:", x, *y + 16.0, 12.0, Color::WHITE);
            let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            let (new_name, _) = TextInput::new(
                self.properties_panel.widget_ids.menu_name_id,
                field_rect,
                &current_name,
            )
            .blocked(blocked)
            .show(ctx);

            if new_name != current_name && !new_name.is_empty() {
                let is_duplicate = self.templates.iter().any(|t| t.id == new_name);
                if !is_duplicate {
                    if let Some(template) = self.current_template_mut() {
                        template.id = new_name;
                    }
                }
            }
        }
        *y += ROW_HEIGHT;

        // Mode dropdown
        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Mode:", x, *y + 16.0, 12.0, Color::WHITE);
            let mode_options = ["Paused", "Overlay"];
            let current_mode_str = match current_mode {
                MenuMode::Paused => "Paused",
                MenuMode::Overlay => "Overlay",
            };
            let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            if let Some(selected) = Dropdown::new(
                self.properties_panel.widget_ids.mode_id,
                dropdown_rect,
                current_mode_str,
                &mode_options,
                |s| s.to_string(),
            )
            .blocked(blocked)
            .fixed_width()
            .show(ctx)
            {
                let new_mode = match selected {
                    "Paused" => MenuMode::Paused,
                    "Overlay" => MenuMode::Overlay,
                    _ => current_mode,
                };
                if let Some(template) = self.current_template_mut() {
                    template.mode = new_mode;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Background section
        *y += 4.0;
        if row_visible(*y, 20.0, clip) {
            ctx.draw_text("Background", x, *y + 14.0, 12.0, Color::GREY);
        }
        *y += 20.0;

        if row_visible(*y, ROW_HEIGHT, clip) {
            ctx.draw_text("Type:", x, *y + 16.0, 12.0, Color::WHITE);
            let bg_options = ["None", "Solid Color", "Dimmed"];
            let current_bg_str = match current_bg {
                MenuBackground::None => "None",
                MenuBackground::SolidColor(_) => "Solid Color",
                MenuBackground::Dimmed(_) => "Dimmed",
            };
            let dropdown_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
            if let Some(selected) = Dropdown::new(
                self.properties_panel.widget_ids.bg_type_id,
                dropdown_rect,
                current_bg_str,
                &bg_options,
                |s| s.to_string(),
            )
            .blocked(blocked)
            .fixed_width()
            .show(ctx)
            {
                let new_bg = match selected {
                    "None" => MenuBackground::None,
                    "Solid Color" => MenuBackground::SolidColor(Color::BLACK),
                    "Dimmed" => MenuBackground::Dimmed(0.7),
                    _ => current_bg,
                };
                if let Some(template) = self.current_template_mut() {
                    template.background = new_bg;
                }
            }
        }
        *y += ROW_HEIGHT;

        // Conditional fields based on background type
        match current_bg {
            MenuBackground::SolidColor(color) => {
                if row_visible(*y, ROW_HEIGHT, clip) {
                    ctx.draw_text("Color:", x, *y + 16.0, 12.0, Color::WHITE);
                    let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
                    let new_color = ColorInput::new(
                        self.properties_panel.widget_ids.bg_color_id,
                        field_rect,
                        color,
                    )
                    .blocked(blocked)
                    .show(ctx);
                    if new_color != color {
                        if let Some(template) = self.current_template_mut() {
                            template.background = MenuBackground::SolidColor(new_color);
                        }
                    }
                }
                *y += ROW_HEIGHT;
            }
            MenuBackground::Dimmed(alpha) => {
                if row_visible(*y, ROW_HEIGHT, clip) {
                    ctx.draw_text("Alpha:", x, *y + 16.0, 12.0, Color::WHITE);
                    let field_rect = Rect::new(x + LABEL_WIDTH, *y, w - LABEL_WIDTH, FIELD_HEIGHT);
                    let (new_alpha, changed) = gui_slider(
                        ctx,
                        self.properties_panel.widget_ids.bg_alpha_id,
                        field_rect,
                        0.0,
                        1.0,
                        alpha,
                    );
                    if changed {
                        if let Some(template) = self.current_template_mut() {
                            template.background = MenuBackground::Dimmed(new_alpha);
                        }
                    }
                }
                *y += ROW_HEIGHT;
            }
            MenuBackground::None => {}
        }
    }
}
