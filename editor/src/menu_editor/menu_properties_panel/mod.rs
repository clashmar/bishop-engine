// editor/src/menu_editor/menu_properties_panel/mod.rs
mod menu_properties;
mod element_properties;
mod layout_properties;
mod common_properties;

use crate::menu_editor::MenuEditor;
use engine_core::prelude::*;
use bishop::prelude::*;

pub(crate) const ROW_HEIGHT: f32 = 28.0;
pub(crate) const LABEL_WIDTH: f32 = 80.0;
pub(crate) const FIELD_HEIGHT: f32 = 24.0;

/// Widget IDs for the properties module.
#[derive(Default)]
pub struct PropertiesWidgetIds {
    pub(crate) name_id: WidgetId,
    pub(crate) text_id: WidgetId,
    pub(crate) font_size_id: WidgetId,
    pub(crate) action_id: WidgetId,
    pub(crate) action_param_id: WidgetId,
    pub(crate) z_order_id: WidgetId,
    pub(crate) pos_x_id: WidgetId,
    pub(crate) pos_y_id: WidgetId,
    pub(crate) size_w_id: WidgetId,
    pub(crate) size_h_id: WidgetId,
    pub(crate) nav_up_id: WidgetId,
    pub(crate) nav_down_id: WidgetId,
    pub(crate) nav_left_id: WidgetId,
    pub(crate) nav_right_id: WidgetId,
    pub(crate) layout_direction_id: WidgetId,
    pub(crate) layout_grid_cols_id: WidgetId,
    pub(crate) layout_spacing_id: WidgetId,
    pub(crate) layout_pad_top_id: WidgetId,
    pub(crate) layout_pad_right_id: WidgetId,
    pub(crate) layout_pad_bottom_id: WidgetId,
    pub(crate) layout_pad_left_id: WidgetId,
    pub(crate) layout_h_align_id: WidgetId,
    pub(crate) layout_v_align_id: WidgetId,
    pub(crate) layout_item_w_id: WidgetId,
    pub(crate) layout_item_h_id: WidgetId,
    pub(crate) layout_nav_up_id: WidgetId,
    pub(crate) layout_nav_down_id: WidgetId,
    pub(crate) layout_nav_left_id: WidgetId,
    pub(crate) layout_nav_right_id: WidgetId,
    pub(crate) label_h_align_id: WidgetId,
    pub(crate) bg_type_id: WidgetId,
    pub(crate) bg_color_id: WidgetId,
    pub(crate) bg_alpha_id: WidgetId,
    pub(crate) mode_id: WidgetId,
}

/// Groups property panel data.
pub struct MenuPropertiesPanel {
    pub(crate) scroll_state: ScrollState,
    pub(crate) widget_ids: PropertiesWidgetIds,
}

impl MenuPropertiesPanel {
    /// Creates a new properties panel.
    pub fn new() -> Self {
        Self {
            scroll_state: ScrollState::new(),
            widget_ids: PropertiesWidgetIds::default(),
        }
    }
}

impl Default for MenuPropertiesPanel {
    fn default() -> Self {
        Self::new()
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

        let area = ScrollableArea::new(rect, content_height)
            .scroll_speed(20.0)
            .blocked(blocked)
            .begin(ctx, &mut self.properties_panel.scroll_state);

        let mut y = rect.y + self.properties_panel.scroll_state.scroll_y + 8.0;
        let content_x = rect.x + 8.0;
        let content_w = area.content_rect().w - 16.0;

        if area.is_fully_visible(y, 24.0) {
            ctx.draw_text("Properties", content_x, y + 14.0, 14.0, Color::GREY);
        }
        y += 24.0;

        if self.selected_element_index.is_none() {
            self.draw_menu_properties(ctx, &mut y, content_x, content_w, blocked, &rect);
            area.draw_scrollbar(ctx, self.properties_panel.scroll_state.scroll_y);
            return;
        }

        let element_kind = self
            .selected_element()
            .map(|e| e.kind.clone());

        let Some(kind) = element_kind else {
            area.draw_scrollbar(ctx, self.properties_panel.scroll_state.scroll_y);
            return;
        };

        self.draw_common_properties(ctx, &mut y, content_x, content_w, blocked, &rect);
        y += 8.0;

        match kind {
            MenuElementKind::Label(_) => {
                self.draw_label_properties(ctx, &mut y, content_x, content_w, blocked, &rect);
            }
            MenuElementKind::Button(_) => {
                self.draw_button_properties(ctx, &mut y, content_x, content_w, blocked, &rect);
            }
            MenuElementKind::Panel(_) => {
                self.draw_panel_properties(ctx, &mut y, content_x, content_w, blocked, &rect);
            }
            MenuElementKind::LayoutGroup(_) => {
                self.draw_layout_group_properties(ctx, &mut y, content_x, content_w, blocked, &rect);
            }
        }

        area.draw_scrollbar(ctx, self.properties_panel.scroll_state.scroll_y);
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
                let nav_height = if self.selected_child_index.is_none() {
                    28.0 + ROW_HEIGHT * 4.0
                } else {
                    0.0
                };
                ROW_HEIGHT * 3.0 + param_row + nav_height
            }
            Some(MenuElementKind::Panel(_)) => 0.0,
            Some(MenuElementKind::LayoutGroup(group)) => {
                let grid_row = if matches!(group.layout.direction, LayoutDirection::Grid { .. }) {
                    ROW_HEIGHT
                } else {
                    0.0
                };
                ROW_HEIGHT * (1.0 + 1.0 + 4.0 + 2.0 + 2.0)
                    + grid_row
                    + 20.0 * 3.0 // section headers
                    + 4.0 * 3.0  // section gaps
                    + 20.0 // children header
                    + ROW_HEIGHT * group.children.len() as f32
                    + 8.0 + 20.0 // navigation section header
                    + ROW_HEIGHT * 4.0 // nav dropdowns
            }
            None => {
                let mut h = 20.0 + ROW_HEIGHT + 4.0 + 20.0 + ROW_HEIGHT;
                if let Some(template) = self.current_template() {
                    if !matches!(template.background, MenuBackground::None) {
                        h += ROW_HEIGHT;
                    }
                }
                return base_height + h;
            }
        };

        let pos_size_height = if self.is_selected_child_managed() {
            20.0
        } else {
            ROW_HEIGHT * 2.0 + 20.0 + 8.0
        };

        let z_order_height = if self.selected_child_index.is_none() { ROW_HEIGHT } else { 0.0 };
        let common_height = ROW_HEIGHT * 2.0 + z_order_height + ROW_HEIGHT * 2.0 + pos_size_height + 8.0;

        base_height + element_height + common_height
    }
}
