use crate::menu::*;
use serde::{Deserialize, Serialize};
use bishop::prelude::*;

/// Serializable menu definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTemplate {
    pub id: String,
    pub background: MenuBackground,
    pub elements: Vec<MenuElement>,
    pub mode: MenuMode,
}

impl MenuTemplate {
    /// Creates a new menu template.
    pub fn new(id: String) -> Self {
        Self {
            id,
            background: MenuBackground::default(),
            elements: Vec::new(),
            mode: MenuMode::Paused,
        }
    }

    /// Renders the menu background over the given viewport rect.
    pub fn render_background<C: BishopContext>(&self, ctx: &mut C, viewport: Rect) {
        match self.background {
            MenuBackground::None => {}
            MenuBackground::SolidColor(color) => {
                ctx.draw_rectangle(viewport.x, viewport.y, viewport.w, viewport.h, color);
            }
            MenuBackground::Dimmed(alpha) => {
                ctx.draw_rectangle(viewport.x, viewport.y, viewport.w, viewport.h, Color::new(0.0, 0.0, 0.0, alpha));
            }
        }
    }

    /// Returns element indices sorted by z_order (stable, ascending).
    pub fn sorted_element_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.elements.len()).collect();
        indices.sort_by_key(|&i| self.elements[i].z_order);
        indices
    }

    /// Renders menu labels transformed from normalized to screen-space using canvas origin/size.
    pub fn render_labels<C: BishopContext>(&self, ctx: &mut C, canvas_origin: Vec2, canvas_size: Vec2) {
        for i in self.sorted_element_indices() {
            let element = &self.elements[i];
            if !element.visible {
                continue;
            }
            match &element.kind {
                MenuElementKind::Label(label) => {
                    let screen_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                    Self::render_label(ctx, label, screen_rect);
                }
                MenuElementKind::LayoutGroup(group) => {
                    let resolved = resolve_layout(group, element.rect);
                    for (child, rect) in group.children.iter().zip(resolved.iter()) {
                        if !child.element.visible {
                            continue;
                        }
                        if let MenuElementKind::Label(label) = &child.element.kind {
                            let screen_rect = normalized_rect_to_screen(*rect, canvas_origin, canvas_size);
                            Self::render_label(ctx, label, screen_rect);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn render_label<C: BishopContext>(ctx: &mut C, label: &LabelElement, rect: Rect) {
        let txt_dims = ctx.measure_text(&label.text, label.font_size);
        let txt_x = rect.x + (rect.w - txt_dims.width) / 2.0;
        let txt_y = rect.y + (rect.h - txt_dims.height) / 2.0 + txt_dims.offset_y;
        ctx.draw_text(&label.text, txt_x, txt_y, label.font_size, label.color);
    }

    /// Returns button elements in z_order with their data.
    pub fn buttons(&self) -> Vec<(&ButtonElement, Rect, bool)> {
        let mut result = Vec::new();
        for i in self.sorted_element_indices() {
            let element = &self.elements[i];
            match &element.kind {
                MenuElementKind::Button(button) => {
                    result.push((button, element.rect, element.enabled));
                }
                MenuElementKind::LayoutGroup(group) => {
                    let resolved = resolve_layout(group, element.rect);
                    for (child, rect) in group.children.iter().zip(resolved.iter()) {
                        if let MenuElementKind::Button(button) = &child.element.kind {
                            result.push((button, *rect, child.element.enabled));
                        }
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// Returns the number of focusable elements.
    pub fn focusable_count(&self) -> usize {
        self.count_focusable_in(&self.elements)
    }

    fn count_focusable_in(&self, elements: &[MenuElement]) -> usize {
        let mut count = 0;
        for element in elements {
            if !element.enabled || !element.visible {
                continue;
            }
            match &element.kind {
                MenuElementKind::Button(_) => count += 1,
                MenuElementKind::LayoutGroup(group) => {
                    for child in &group.children {
                        if matches!(child.element.kind, MenuElementKind::Button(_))
                            && child.element.enabled
                            && child.element.visible
                        {
                            count += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        count
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
            if let MenuElementKind::LayoutGroup(group) = &element.kind {
                for child in &group.children {
                    if matches!(child.element.kind, MenuElementKind::Button(_))
                        && child.element.enabled
                        && child.element.visible
                    {
                        if current_index == focus_index {
                            return Some(&child.element);
                        }
                        current_index += 1;
                    }
                }
            }
        }
        None
    }
}
