use bishop::prelude::*;
use widgets::*;
use crate::menu::menu_mode::MenuMode;
use crate::menu::menu_builder::*;

/// Manages the current menu state and active menu.
pub struct MenuManager {
    /// Current menu mode.
    pub mode: MenuMode,
    /// The currently active menu, if any.
    active_menu: Option<Menu>,
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuManager {
    /// Creates a new menu manager in running state.
    pub fn new() -> Self {
        Self {
            mode: MenuMode::Running,
            active_menu: None,
        }
    }

    /// Returns true if the menu is blocking game updates.
    pub fn is_pausing_game(&self) -> bool {
        !self.mode.is_game_running()
    }

    /// Returns true if the menu is hiding the game.
    pub fn is_hiding_game(&self) -> bool {
        !self.mode.is_game_visible()
    }

    /// Handles input for menu toggling and interaction.
    pub fn handle_input<C: BishopContext>(&mut self, ctx: &mut C) {
        if ctx.is_key_pressed(KeyCode::P) {
            match self.mode {
                MenuMode::Running => self.open_pause_menu(ctx),
                MenuMode::Paused => self.close(),
                _ => {}
            }
        }

        if ctx.is_key_pressed(KeyCode::Escape) && self.mode == MenuMode::Paused {
            self.close();
        }
    }

    /// Opens the pause menu.
    pub fn open_pause_menu<C: BishopContext>(&mut self, ctx: &mut C) {
        let w = ctx.screen_width();
        let h = ctx.screen_height();
        self.active_menu = Some(Self::create_pause_menu(w, h));
        self.mode = MenuMode::Paused;
    }

    /// Closes any active menu and resumes the game.
    pub fn close(&mut self) {
        self.active_menu = None;
        self.mode = MenuMode::Running;
    }

    /// Renders the active menu if any.
    pub fn render<C: BishopContext>(&mut self, ctx: &mut C) {
        widgets_frame_start(ctx);

        if let Some(menu) = &self.active_menu {
            if let Some(action) = menu.render(ctx) {
                self.handle_action(action);
            }
        }

        widgets_frame_end(ctx);
    }

    fn handle_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::Resume => self.close(),
            MenuAction::Custom(_) => {}
        }
    }

    /// Creates the default pause menu.
    fn create_pause_menu(screen_w: f32, screen_h: f32) -> Menu {
        MenuBuilder::new(screen_w, screen_h)
            .background(MenuBackground::Dimmed(0.7))
            .label("PAUSED")
            .spacer(8.0)
            .button("Resume", MenuAction::Resume)
            .build()
    }
}
