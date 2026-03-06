use bishop::prelude::*;
use serde::{Deserialize, Serialize};
use crate::menu::{
    MenuBackground, MenuElement, MenuElementKind, MenuMode, MenuTemplate,
    layout::{LayoutConfig, LayoutDirection},
};

/// Actions that can be triggered by menu items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MenuAction {
    /// Resume the game.
    Resume,
    /// Open a menu by id.
    OpenMenu(String),
    /// Close the current menu.
    CloseMenu,
    /// Quit to main menu.
    QuitToMainMenu,
    /// Quit the game.
    QuitGame,
    /// Custom action for extensibility.
    Custom(String),
}

/// Legacy menu item type for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MenuItem {
    /// Static text label.
    Label { text: String, rect: Rect },
    /// Clickable button.
    Button { text: String, rect: Rect, action: MenuAction },
    /// Vertical spacing.
    Spacer { height: f32 },
}

/// Legacy menu type for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub background: MenuBackground,
}

impl Menu {
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
        for item in &self.items {
            if let MenuItem::Label { text, rect } = item {
                let txt_dims = ctx.measure_text(text, 24.0);
                let txt_x = rect.x + (rect.w - txt_dims.width) / 2.0;
                let txt_y = rect.y + rect.h * 0.7;
                ctx.draw_text(text, txt_x, txt_y, 24.0, Color::WHITE);
            }
        }
    }

    /// Returns an iterator over button items.
    pub fn buttons(&self) -> impl Iterator<Item = (&str, Rect, &MenuAction)> {
        self.items.iter().filter_map(|item| {
            if let MenuItem::Button { text, rect, action } = item {
                Some((text.as_str(), *rect, action))
            } else {
                None
            }
        })
    }
}

/// Builder for composing menus with flexible layouts.
#[derive(Debug, Clone)]
pub struct MenuBuilder {
    id: String,
    elements: Vec<MenuElement>,
    background: MenuBackground,
    layout: LayoutConfig,
    mode: MenuMode,
    screen_width: f32,
    screen_height: f32,
    y_cursor: f32,
    x_cursor: f32,
}

impl MenuBuilder {
    /// Creates a new menu builder with an id.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            elements: Vec::new(),
            background: MenuBackground::default(),
            layout: LayoutConfig::default(),
            mode: MenuMode::Paused,
            screen_width: 800.0,
            screen_height: 600.0,
            y_cursor: 0.0,
            x_cursor: 0.0,
        }
    }

    /// Creates a new menu builder for legacy compatibility.
    pub fn new_legacy(screen_width: f32, screen_height: f32) -> Self {
        let item_width = 200.0;
        let center_x = (screen_width - item_width) / 2.0;
        let y_cursor = screen_height / 3.0;

        Self {
            id: "legacy".to_string(),
            elements: Vec::new(),
            background: MenuBackground::default(),
            layout: LayoutConfig {
                item_width,
                item_height: 40.0,
                spacing: 16.0,
                ..Default::default()
            },
            mode: MenuMode::Paused,
            screen_width,
            screen_height,
            y_cursor,
            x_cursor: center_x,
        }
    }

    /// Sets the screen dimensions for layout calculations.
    pub fn screen_size(mut self, width: f32, height: f32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    /// Sets the menu mode.
    pub fn mode(mut self, mode: MenuMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the background style.
    pub fn background(mut self, bg: MenuBackground) -> Self {
        self.background = bg;
        self
    }

    /// Sets vertical layout direction.
    pub fn vertical(mut self) -> Self {
        self.layout.direction = LayoutDirection::Vertical;
        self
    }

    /// Sets horizontal layout direction.
    pub fn horizontal(mut self) -> Self {
        self.layout.direction = LayoutDirection::Horizontal;
        self
    }

    /// Sets grid layout direction with specified columns.
    pub fn grid(mut self, columns: u32) -> Self {
        self.layout.direction = LayoutDirection::Grid { columns };
        self
    }

    /// Sets the layout configuration.
    pub fn with_layout(mut self, layout: LayoutConfig) -> Self {
        self.layout = layout;
        self
    }

    /// Adds a text label.
    pub fn label(mut self, text: &str) -> Self {
        let rect = self.compute_next_rect();
        self.elements.push(MenuElement::label(text.to_string(), rect));
        self.advance_cursor();
        self
    }

    /// Adds a clickable button.
    pub fn button(mut self, text: &str, action: MenuAction) -> Self {
        let rect = self.compute_next_rect();
        self.elements.push(MenuElement::button(text.to_string(), action, rect));
        self.advance_cursor();
        self
    }

    /// Adds vertical or horizontal spacing.
    pub fn spacer(mut self, size: f32) -> Self {
        let rect = self.compute_spacer_rect(size);
        self.elements.push(MenuElement::spacer(size, rect));
        self.advance_cursor_by(size);
        self
    }

    /// Adds a nested panel with its own layout.
    pub fn panel<F>(mut self, build_fn: F) -> Self
    where
        F: FnOnce(MenuBuilder) -> MenuBuilder,
    {
        let panel_builder = MenuBuilder::new("panel").screen_size(self.screen_width, self.screen_height);
        let panel_builder = build_fn(panel_builder);

        let rect = self.compute_next_rect();
        self.elements.push(MenuElement::panel(panel_builder.elements, rect));
        self.advance_cursor();
        self
    }

    /// Builds the final menu template.
    pub fn build(self) -> MenuTemplate {
        MenuTemplate {
            id: self.id,
            layout: self.layout,
            background: self.background,
            elements: self.elements,
            mode: self.mode,
        }
    }

    /// Legacy build method that returns Menu for backward compatibility.
    pub fn build_legacy(self) -> Menu {
        let items = self.elements.into_iter().map(|element| {
            match element.kind {
                MenuElementKind::Label(label) => MenuItem::Label {
                    text: label.text,
                    rect: element.rect,
                },
                MenuElementKind::Button(button) => MenuItem::Button {
                    text: button.text,
                    rect: element.rect,
                    action: button.action,
                },
                MenuElementKind::Spacer(spacer) => MenuItem::Spacer {
                    height: spacer.size,
                },
                MenuElementKind::Panel(_) => MenuItem::Spacer { height: 0.0 },
            }
        }).collect();

        Menu {
            items,
            background: self.background,
        }
    }

    fn compute_next_rect(&self) -> Rect {
        match self.layout.direction {
            LayoutDirection::Vertical => {
                let x = self.x_cursor.max((self.screen_width - self.layout.item_width) / 2.0);
                Rect::new(x, self.y_cursor, self.layout.item_width, self.layout.item_height)
            }
            LayoutDirection::Horizontal => {
                Rect::new(self.x_cursor, self.y_cursor, self.layout.item_width, self.layout.item_height)
            }
            LayoutDirection::Grid { .. } => {
                Rect::new(self.x_cursor, self.y_cursor, self.layout.item_width, self.layout.item_height)
            }
        }
    }

    fn compute_spacer_rect(&self, size: f32) -> Rect {
        match self.layout.direction {
            LayoutDirection::Vertical => {
                Rect::new(self.x_cursor, self.y_cursor, self.layout.item_width, size)
            }
            LayoutDirection::Horizontal => {
                Rect::new(self.x_cursor, self.y_cursor, size, self.layout.item_height)
            }
            LayoutDirection::Grid { .. } => {
                Rect::new(self.x_cursor, self.y_cursor, size, size)
            }
        }
    }

    fn advance_cursor(&mut self) {
        match self.layout.direction {
            LayoutDirection::Vertical => {
                self.y_cursor += self.layout.item_height + self.layout.spacing;
            }
            LayoutDirection::Horizontal => {
                self.x_cursor += self.layout.item_width + self.layout.spacing;
            }
            LayoutDirection::Grid { columns } => {
                self.x_cursor += self.layout.item_width + self.layout.spacing;
                let current_col = (self.elements.len() as u32) % columns;
                if current_col == 0 {
                    self.x_cursor = 0.0;
                    self.y_cursor += self.layout.item_height + self.layout.spacing;
                }
            }
        }
    }

    fn advance_cursor_by(&mut self, amount: f32) {
        match self.layout.direction {
            LayoutDirection::Vertical => {
                self.y_cursor += amount;
            }
            LayoutDirection::Horizontal => {
                self.x_cursor += amount;
            }
            LayoutDirection::Grid { .. } => {
                self.y_cursor += amount;
            }
        }
    }
}
