// editor/src/menu_editor/menu_properties_panel/common_properties.rs
use crate::menu_editor::MenuEditor;
use super::{ROW_HEIGHT, LABEL_WIDTH, FIELD_HEIGHT};
use engine_core::prelude::*;
use bishop::prelude::*;

impl MenuEditor {
    pub(super) fn draw_common_properties(
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
}
