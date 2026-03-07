use crate::menu::*;
use serde::{Deserialize, Serialize};
use bishop::prelude::*;

/// Serializable menu definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTemplate {
    pub id: String,
    pub layout: LayoutConfig,
    pub background: MenuBackground,
    pub elements: Vec<MenuElement>,
    pub mode: MenuMode,
}

impl MenuTemplate {
    /// Creates a new menu template.
    pub fn new(id: String) -> Self {
        Self {
            id,
            layout: LayoutConfig::default(),
            background: MenuBackground::default(),
            elements: Vec::new(),
            mode: MenuMode::Paused,
        }
    }

    /// Renders the menu background.
    pub fn render_background<C: BishopContext>(&self, ctx: &mut C) {
        let w = ctx.screen_width();
        let h = ctx.screen_height();

        match self.background {
            MenuBackground::None => {}
            MenuBackground::SolidColor(color) => {
                ctx.draw_rectangle(0.0, 0.0, w, h, color);
            }
            MenuBackground::Dimmed(alpha) => {
                ctx.draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, alpha));
            }
        }
    }

    /// Renders menu labels.
    pub fn render_labels<C: BishopContext>(&self, ctx: &mut C) {
        for element in &self.elements {
            if !element.visible {
                continue;
            }
            if let MenuElementKind::Label(label) = &element.kind {
                self.render_label(ctx, label, element.rect);
            }
        }
    }

    fn render_label<C: BishopContext>(&self, ctx: &mut C, label: &LabelElement, rect: Rect) {
        let txt_dims = ctx.measure_text(&label.text, label.font_size);
        let txt_x = rect.x + (rect.w - txt_dims.width) / 2.0;
        let txt_y = rect.y + rect.h * 0.7;
        ctx.draw_text(&label.text, txt_x, txt_y, label.font_size, label.color);
    }

    /// Returns an iterator over button elements with their data.
    pub fn buttons(&self) -> impl Iterator<Item = (&ButtonElement, Rect, bool)> {
        self.elements.iter().filter_map(|element| {
            if let MenuElementKind::Button(button) = &element.kind {
                Some((button, element.rect, element.enabled))
            } else {
                None
            }
        })
    }

    /// Returns the number of focusable elements.
    pub fn focusable_count(&self) -> usize {
        self.elements
            .iter()
            .filter(|e| matches!(e.kind, MenuElementKind::Button(_)) && e.enabled && e.visible)
            .count()
    }

    /// Gets the button at the given focus index.
    pub fn get_focused_button(&self, focus_index: usize) -> Option<&MenuElement> {
        let mut current_index = 0;
        for element in &self.elements {
            if matches!(element.kind, MenuElementKind::Button(_)) && element.enabled && element.visible {
                if current_index == focus_index {
                    return Some(element);
                }
                current_index += 1;
            }
        }
        None
    }
}
