use crate::menu::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bishop::prelude::*;
use widgets::*;

/// Manages menu templates, active menu stack, and navigation.
pub struct MenuManager {
    /// Registered menu templates by id.
    templates: HashMap<String, MenuTemplate>,
    /// Stack of active menu ids (top = current).
    menu_stack: Vec<String>,
    /// Navigation input bindings.
    navigation: MenuNavigation,
    /// Current focus index for keyboard navigation.
    focus_index: usize,
    /// Action handler for custom menu actions.
    action_handler: Box<dyn MenuActionHandler>,
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the menu mode for a given menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MenuMode {
    #[default]
    /// Game paused, visible in background.
    Paused,
    /// Full black screen, game hidden.
    BlackScreen,
    /// Overlay menu, game continues.
    Overlay,
    /// Menu takes up full dimensions of screen.
    FullScreen,
}

impl MenuMode {
    /// Returns true if the game logic should be pause.
    pub fn is_paused(&self) -> bool {
        matches!(self, MenuMode::Paused | MenuMode::BlackScreen | MenuMode::FullScreen)
    }

    /// Returns true if the game is hidden by a menu.
    pub fn is_hiding_game(&self) -> bool {
        matches!(self, MenuMode::BlackScreen | MenuMode::FullScreen)
    }
}

impl MenuManager {
    /// Creates a new menu manager with default settings.
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            menu_stack: Vec::new(),
            navigation: MenuNavigation::default(),
            focus_index: 0,
            action_handler: Box::new(NoOpActionHandler),
        };
        manager.register_default_menus();
        manager
    }

    /// Sets the custom action handler.
    pub fn set_action_handler<H: MenuActionHandler + 'static>(&mut self, handler: H) {
        self.action_handler = Box::new(handler);
    }

    /// Registers a menu template.
    pub fn register_template(&mut self, template: MenuTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Opens a menu by id.
    pub fn open_menu(&mut self, id: &str) {
        if self.templates.contains_key(id) {
            self.menu_stack.push(id.to_string());
            self.focus_index = 0;
        }
    }

    /// Closes the current menu and returns to previous menu if any.
    pub fn close_menu(&mut self) {
        self.menu_stack.pop();
        self.focus_index = 0;
    }

    /// Closes all menus and returns to game.
    pub fn close_all(&mut self) {
        self.menu_stack.clear();
        self.focus_index = 0;
    }

    /// Returns the current menu mode based on active menu.
    pub fn mode(&self) -> Option<MenuMode> {
        if let Some(menu_id) = self.menu_stack.last() {
            if let Some(template) = self.templates.get(menu_id) {
                return Some(template.mode);
            }
        }
        None
    }

    /// Returns true if the menu is blocking game updates.
    pub fn is_pausing_game(&self) -> bool {
        self.mode().map_or(false, |m| m.is_paused())
    }

    /// Returns true if the menu is hiding the game.
    pub fn is_hiding_game(&self) -> bool {
        self.mode().map_or(false, |m| m.is_hiding_game())
    }

    /// Returns true if any menu is active.
    pub fn has_active_menu(&self) -> bool {
        !self.menu_stack.is_empty()
    }

    /// Handles input for menu toggling and navigation.
    pub fn handle_input<C: BishopContext>(&mut self, ctx: &mut C) {
        if self.navigation.pause_pressed(ctx) {
            if self.has_active_menu() {
                self.close_menu();
            } else {
                self.open_menu("pause");
            }
            return;
        }

        if !self.has_active_menu() {
            return;
        }

        if let Some(menu_id) = self.menu_stack.last().cloned() {
            if let Some(template) = self.templates.get(&menu_id) {
                let focusable_count = template.focusable_count();
                if focusable_count == 0 {
                    return;
                }

                if self.navigation.up_pressed(ctx) && self.focus_index > 0 {
                    self.focus_index -= 1;
                }

                if self.navigation.down_pressed(ctx) && self.focus_index < focusable_count - 1 {
                    self.focus_index += 1;
                }

                let cancel_pressed = self.navigation.cancel_pressed(ctx);
                let confirm_pressed = self.navigation.confirm_pressed(ctx);
                let action_to_handle = if confirm_pressed {
                    template.get_focused_button(self.focus_index)
                        .and_then(|element| {
                            if let MenuElementKind::Button(button) = &element.kind {
                                Some(button.action.clone())
                            } else {
                                None
                            }
                        })
                } else {
                    None
                };

                if cancel_pressed {
                    self.close_menu();
                }

                if let Some(action) = action_to_handle {
                    self.handle_action(action);
                }
            }
        }
    }

    /// Renders the active menu if any.
    pub fn render<C: BishopContext>(&mut self, ctx: &mut C) {
        if !self.has_active_menu() {
            return;
        }

        widgets_frame_start(ctx);

        let mut triggered_action = None;

        if let Some(menu_id) = self.menu_stack.last() {
            if let Some(template) = self.templates.get(menu_id) {
                template.render_background(ctx);
                template.render_labels(ctx);

                let mut button_index = 0;
                for (button, rect, enabled) in template.buttons() {
                    let _is_focused = button_index == self.focus_index;
                    let btn = Button::new(rect, &button.text)
                        .blocked(!enabled);

                    if btn.show(ctx) {
                        triggered_action = Some(button.action.clone());
                    }

                    button_index += 1;
                }
            }
        }

        widgets_frame_end(ctx);

        if let Some(action) = triggered_action {
            self.handle_action(action);
        }
    }

    fn handle_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::Resume => self.close_all(),
            MenuAction::CloseMenu => self.close_menu(),
            MenuAction::OpenMenu(menu_id) => self.open_menu(&menu_id),
            MenuAction::QuitToMainMenu => {
                self.close_all();
            }
            MenuAction::QuitGame => {
                self.close_all();
            }
            MenuAction::Custom(action_name) => {
                self.action_handler.handle_action(&action_name);
            }
        }
    }

    fn register_default_menus(&mut self) {
        let pause_menu = MenuBuilder::new("pause")
            .screen_size(800.0, 600.0)
            .background(MenuBackground::Dimmed(0.7))
            .vertical()
            .label("PAUSED")
            .spacer(8.0)
            .button("Resume", MenuAction::Resume)
            .build();

        self.register_template(pause_menu);
    }
}

/// Legacy support functions.
impl MenuManager {
    /// Opens the pause menu using legacy method.
    pub fn open_pause_menu<C: BishopContext>(&mut self, ctx: &mut C) {
        let w = ctx.screen_width();
        let h = ctx.screen_height();
        let pause_template = MenuBuilder::new("pause")
            .screen_size(w, h)
            .background(MenuBackground::Dimmed(0.7))
            .vertical()
            .label("PAUSED")
            .spacer(8.0)
            .button("Resume", MenuAction::Resume)
            .build();
        self.register_template(pause_template);
        self.open_menu("pause");
    }

    /// Closes any active menu and resumes the game.
    pub fn close(&mut self) {
        self.close_all();
    }
}
