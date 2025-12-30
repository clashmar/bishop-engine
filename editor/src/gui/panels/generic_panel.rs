// editor/src/gui/generic_panel.rs
use crate::gui::gui_constants::*;
use crate::Editor;
use engine_core::ui::widgets::*;
use engine_core::ui::text::*;
use macroquad::prelude::*;

/// Must be globally unique.
pub type PanelId = &'static str;

/// Defines the features and content of the panel.
pub trait PanelDefinition {
    /// Unique title (also used as id).
    fn title(&self) -> &'static str;
    /// Default rect when first created.
    fn default_rect(&self) -> Rect;
    /// Draws panel contents.
    fn draw(&mut self, rect: Rect, editor: &mut Editor);
}

/// Movable and collabsible panel to be composed with the supplied `PanelDefinition`.
pub struct GenericPanel {
    pub title: &'static str,
    pub rect: Rect,
    pub visible: bool,
    pub collapsed: bool,
    dragging: bool,
    drag_offset: Vec2,
    definition: Box<dyn PanelDefinition>,
}

impl GenericPanel {
    pub fn new(definition: impl PanelDefinition + 'static) -> Self {
        let title = definition.title();
        let rect = definition.default_rect();

        Self {
            title,
            rect,
            visible: true,
            collapsed: false,
            dragging: false,
            drag_offset: Vec2::ZERO,
            definition: Box::new(definition),
        }
    }

    pub fn update_and_draw(&mut self, editor: &mut Editor) {
        if !self.visible {
            return;
        }

        // Take a snapshot of the rect before it mutates
        let panel_rect = self.rect;

        let title_bar_h = 28.0;
        let title_bar = Rect::new(panel_rect.x, panel_rect.y, panel_rect.w, title_bar_h);

        // Title bar
        draw_rectangle(title_bar.x, title_bar.y, title_bar.w, title_bar.h, PANEL_COLOR);

        // Collapse button
        let collapse_rect = Rect::new(panel_rect.left() + 5., panel_rect.y + 4., 20., 20.);
        if gui_button_plain_default(collapse_rect, if self.collapsed { "→" } else { "↓" }, BLACK) {
            self.collapsed = !self.collapsed;
        }

        // Title
        draw_text_ui(self.title, collapse_rect.x + 25., title_bar.y + 20., 16., BLACK);

        // Close button
        let close_rect = Rect::new(panel_rect.right() - 26., panel_rect.y + 4., 20., 20.);
        if gui_button_plain_default(close_rect, "x", BLACK) {
            self.visible = false;
        }

        if self.collapsed {
            return;
        }

        // Content area
        let content_rect = Rect::new(
            panel_rect.x,
            panel_rect.y + title_bar_h,
            panel_rect.w,
            panel_rect.h - title_bar_h,
        );

        // Background
        draw_rectangle(content_rect.x, content_rect.y, content_rect.w, content_rect.h, FIELD_BACKGROUND_COLOR);
        draw_rectangle_lines(content_rect.x, content_rect.y, content_rect.w, content_rect.h, 2., WHITE);

        if !self.collapsed {
            self.definition.draw(content_rect, editor);
        }

        // Drag logic after drawing
        let mouse: Vec2 = mouse_position().into();
        if is_mouse_button_pressed(MouseButton::Left) && title_bar.contains(mouse) {
            self.dragging = true;
            self.drag_offset = mouse - vec2(self.rect.x, self.rect.y);
        }

        if self.dragging {
            if is_mouse_button_down(MouseButton::Left) {
                let new_pos = mouse - self.drag_offset;
                self.rect.x = new_pos.x;
                self.rect.y = new_pos.y;
            } else {
                self.dragging = false;
            }
        }
    }
}
