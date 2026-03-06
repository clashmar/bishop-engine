use bishop::prelude::*;

/// Actions that can be triggered by menu items.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    /// Resume the game.
    Resume,
    /// Custom action for extensibility.
    Custom(String),
}

/// Background style for a menu.
#[derive(Debug, Clone, Copy)]
pub enum MenuBackground {
    /// No background.
    None,
    /// Solid color background.
    SolidColor(Color),
    /// Semi-transparent dimming overlay.
    Dimmed(f32),
}

impl Default for MenuBackground {
    fn default() -> Self {
        MenuBackground::Dimmed(0.7)
    }
}

/// A single item in a menu.
#[derive(Debug, Clone)]
pub enum MenuItem {
    /// Static text label.
    Label { text: String, rect: Rect },
    /// Clickable button.
    Button { text: String, rect: Rect, action: MenuAction },
    /// Vertical spacing.
    Spacer { height: f32 },
}

/// A composed menu with items and background.
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub background: MenuBackground,
}

impl Menu {
    /// Renders the menu and returns any triggered action.
    pub fn render<C: BishopContext>(&self, ctx: &mut C) -> Option<MenuAction> {
        self.render_background(ctx);

        let mut triggered_action = None;

        for item in &self.items {
            match item {
                MenuItem::Label { text, rect } => {
                    let txt_dims = ctx.measure_text(text, 24.0);
                    let txt_x = rect.x + (rect.w - txt_dims.width) / 2.0;
                    let txt_y = rect.y + rect.h * 0.7;
                    ctx.draw_text(text, txt_x, txt_y, 24.0, Color::WHITE);
                }
                MenuItem::Button { text, rect, action } => {
                    if widgets::Button::new(*rect, text).show(ctx) {
                        triggered_action = Some(action.clone());
                    }
                }
                MenuItem::Spacer { .. } => {}
            }
        }

        triggered_action
    }

    fn render_background<C: BishopContext>(&self, ctx: &mut C) {
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
}

/// Builder for composing menus with centered layout.
pub struct MenuBuilder {
    items: Vec<MenuItem>,
    background: MenuBackground,
    y_cursor: f32,
    center_x: f32,
    item_width: f32,
    item_height: f32,
    spacing: f32,
}

impl MenuBuilder {
    /// Creates a new menu builder centered on screen.
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let item_width = 200.0;
        let center_x = (screen_width - item_width) / 2.0;
        let y_cursor = screen_height / 3.0;

        Self {
            items: Vec::new(),
            background: MenuBackground::default(),
            y_cursor,
            center_x,
            item_width,
            item_height: 40.0,
            spacing: 16.0,
        }
    }

    /// Sets the background style.
    pub fn background(mut self, bg: MenuBackground) -> Self {
        self.background = bg;
        self
    }

    /// Adds a text label.
    pub fn label(mut self, text: &str) -> Self {
        let rect = Rect::new(self.center_x, self.y_cursor, self.item_width, self.item_height);
        self.items.push(MenuItem::Label {
            text: text.to_string(),
            rect,
        });
        self.y_cursor += self.item_height + self.spacing;
        self
    }

    /// Adds a clickable button.
    pub fn button(mut self, text: &str, action: MenuAction) -> Self {
        let rect = Rect::new(self.center_x, self.y_cursor, self.item_width, self.item_height);
        self.items.push(MenuItem::Button {
            text: text.to_string(),
            rect,
            action,
        });
        self.y_cursor += self.item_height + self.spacing;
        self
    }

    /// Adds vertical spacing.
    pub fn spacer(mut self, height: f32) -> Self {
        self.items.push(MenuItem::Spacer { height });
        self.y_cursor += height;
        self
    }

    /// Builds the final menu.
    pub fn build(self) -> Menu {
        Menu {
            items: self.items,
            background: self.background,
        }
    }
}
