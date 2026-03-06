// editor/src/menu_editor/properties_module.rs
use crate::menu_editor::MenuEditor;
use engine_core::prelude::*;
use bishop::prelude::*;

const ROW_HEIGHT: f32 = 24.0;

/// Properties editor for the selected menu element.
pub struct PropertiesModule {
    scroll_y: f32,
}

impl PropertiesModule {
    /// Creates a new properties module.
    pub fn new() -> Self {
        Self { scroll_y: 0.0 }
    }

    /// Renders the properties panel.
    pub fn draw(&mut self, ctx: &mut WgpuContext, rect: Rect, menu_editor: &mut MenuEditor, blocked: bool) {
        let mouse: Vec2 = ctx.mouse_position().into();

        if !blocked && rect.contains(mouse) {
            let (_, wheel_y) = ctx.mouse_wheel();
            self.scroll_y += wheel_y * 20.0;
        }

        let content_height = self.calculate_content_height();
        let scroll_range = (content_height - rect.h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        let mut y = rect.y + self.scroll_y + 8.0;

        ctx.draw_text("Properties", rect.x + 8.0, y + 14.0, 14.0, Color::GREY);
        y += 24.0;

        if menu_editor.selected_element_index.is_some() {
            self.draw_element_properties(ctx, rect, &mut y, menu_editor);
        } else {
            ctx.draw_text(
                "No element selected",
                rect.x + 8.0,
                y + 14.0,
                12.0,
                Color::new(0.6, 0.6, 0.6, 1.0),
            );
        }
    }

    fn draw_element_properties(&self, ctx: &mut WgpuContext, rect: Rect, y: &mut f32, menu_editor: &MenuEditor) {
        if let Some(element) = menu_editor.selected_element() {
            match &element.kind {
                MenuElementKind::Label(label) => {
                    self.draw_property_label(ctx, rect, y, "Text:");
                    self.draw_property_label(ctx, rect, y, &label.text);

                    self.draw_property_label(ctx, rect, y, "Font Size:");
                    self.draw_property_label(ctx, rect, y, &format!("{:.1}", label.font_size));
                }
                MenuElementKind::Button(button) => {
                    self.draw_property_label(ctx, rect, y, "Text:");
                    self.draw_property_label(ctx, rect, y, &button.text);

                    self.draw_property_label(ctx, rect, y, "Font Size:");
                    self.draw_property_label(ctx, rect, y, &format!("{:.1}", button.font_size));

                    self.draw_property_label(ctx, rect, y, "Action:");
                    self.draw_property_label(ctx, rect, y, &format!("{:?}", button.action));
                }
                MenuElementKind::Spacer(spacer) => {
                    self.draw_property_label(ctx, rect, y, "Size:");
                    self.draw_property_label(ctx, rect, y, &format!("{:.1}", spacer.size));
                }
                MenuElementKind::Panel(_) => {
                    self.draw_property_label(ctx, rect, y, "Type:");
                    self.draw_property_label(ctx, rect, y, "Panel");
                }
            }

            self.draw_property_label(ctx, rect, y, "Position:");
            self.draw_property_label(ctx, rect, y, &format!("({:.0}, {:.0})", element.rect.x, element.rect.y));

            self.draw_property_label(ctx, rect, y, "Size:");
            self.draw_property_label(ctx, rect, y, &format!("{:.0} x {:.0}", element.rect.w, element.rect.h));
        }
    }

    fn draw_property_label(&self, ctx: &mut WgpuContext, rect: Rect, y: &mut f32, text: &str) {
        if *y < rect.y || *y + ROW_HEIGHT > rect.y + rect.h {
            *y += ROW_HEIGHT;
            return;
        }

        ctx.draw_text(text, rect.x + 8.0, *y + 16.0, 12.0, Color::WHITE);
        *y += ROW_HEIGHT;
    }

    fn calculate_content_height(&self) -> f32 {
        400.0
    }
}

impl Default for PropertiesModule {
    fn default() -> Self {
        Self::new()
    }
}
