// editor/src/menu_editor/menu_list_module.rs
use crate::menu_editor::MenuEditor;
use bishop::prelude::*;
use engine_core::ui::widgets::*;

const MENU_ITEM_HEIGHT: f32 = 24.0;
const BUTTON_HEIGHT: f32 = 28.0;

/// Module for managing the list of menu templates.
pub struct MenuListModule {
    scroll_y: f32,
    new_menu_id: WidgetId,
    pending_new_menu: bool,
    new_menu_name: String,
}

impl MenuListModule {
    /// Creates a new menu list module.
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            new_menu_id: WidgetId::default(),
            pending_new_menu: false,
            new_menu_name: String::new(),
        }
    }

    /// Renders the menu list and handles input.
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

        ctx.draw_text("Menus", content_x, y + 14.0, 14.0, Color::GREY);
        y += 24.0;

        // New/Delete buttons
        let btn_w = (content_w - 8.0) / 2.0;
        let new_btn_rect = Rect::new(content_x, y, btn_w, BUTTON_HEIGHT);
        let delete_btn_rect = Rect::new(content_x + btn_w + 8.0, y, btn_w, BUTTON_HEIGHT);

        let new_clicked = Button::new(new_btn_rect, "New").blocked(blocked).show(ctx);
        let delete_clicked = Button::new(delete_btn_rect, "Delete")
            .blocked(blocked || menu_editor.current_template_index.is_none())
            .show(ctx);

        if new_clicked {
            self.pending_new_menu = true;
            self.new_menu_name = String::new();
        }

        if delete_clicked {
            if let Some(index) = menu_editor.current_template_index {
                menu_editor.delete_template(index);
            }
        }
        y += BUTTON_HEIGHT + 8.0;

        // New menu name input (if pending)
        if self.pending_new_menu {
            let field_rect = Rect::new(content_x, y, content_w - 60.0, 24.0);
            let (new_text, _) = TextInput::new(self.new_menu_id, field_rect, &self.new_menu_name)
                .focused(true)
                .blocked(blocked)
                .show(ctx);
            self.new_menu_name = new_text;

            let ok_rect = Rect::new(content_x + content_w - 50.0, y, 50.0, 24.0);
            let ok_clicked = Button::new(ok_rect, "OK").blocked(blocked).show(ctx);

            if (ok_clicked || ctx.is_key_pressed(KeyCode::Enter)) && !self.new_menu_name.trim().is_empty() {
                let name = self.new_menu_name.trim().to_string();
                menu_editor.create_new_template(name);
                self.pending_new_menu = false;
                self.new_menu_name.clear();
            }

            if ctx.is_key_pressed(KeyCode::Escape) {
                self.pending_new_menu = false;
                self.new_menu_name.clear();
            }

            y += 32.0;
        }

        // List of menus - collect click info first to avoid borrow issues
        let mut clicked_index = None;

        for (index, template) in menu_editor.templates.iter().enumerate() {
            if y < rect.y || y + MENU_ITEM_HEIGHT > rect.y + rect.h {
                y += MENU_ITEM_HEIGHT + 4.0;
                continue;
            }

            let item_rect = Rect::new(content_x, y, content_w, MENU_ITEM_HEIGHT);
            let is_selected = menu_editor.current_template_index == Some(index);
            let hover = item_rect.contains(mouse);

            let bg_color = if is_selected {
                Color::new(0.3, 0.4, 0.6, 1.0)
            } else if hover && !blocked {
                Color::new(0.25, 0.25, 0.3, 1.0)
            } else {
                Color::new(0.2, 0.2, 0.25, 1.0)
            };

            ctx.draw_rectangle(item_rect.x, item_rect.y, item_rect.w, item_rect.h, bg_color);

            ctx.draw_text(
                &template.id,
                item_rect.x + 8.0,
                item_rect.y + 16.0,
                12.0,
                if is_selected { Color::WHITE } else { Color::new(0.8, 0.8, 0.8, 1.0) },
            );

            if hover && !blocked && ctx.is_mouse_button_pressed(MouseButton::Left) {
                clicked_index = Some(index);
            }

            y += MENU_ITEM_HEIGHT + 4.0;
        }

        // Apply clicked selection after iteration completes
        if let Some(index) = clicked_index {
            menu_editor.select_template(index);
        }
    }

    fn calculate_content_height(&self, menu_editor: &MenuEditor) -> f32 {
        let header_height = 24.0;
        let buttons_height = BUTTON_HEIGHT + 8.0;
        let input_height = if self.pending_new_menu { 32.0 } else { 0.0 };
        let items_height = (MENU_ITEM_HEIGHT + 4.0) * menu_editor.templates.len() as f32;

        header_height + buttons_height + input_height + items_height + 16.0
    }
}

impl Default for MenuListModule {
    fn default() -> Self {
        Self::new()
    }
}
