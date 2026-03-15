use crate::menu::*;
use crate::storage::path_utils::menus_folder;
use crate::text::TextManager;
use crate::{onscreen_error, onscreen_log};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bishop::prelude::*;
use widgets::*;
use std::fs;

/// Manages menu templates, active menu stack, and navigation.
pub struct MenuManager {
    /// Registered menu templates by id.
    templates: HashMap<String, MenuTemplate>,
    /// Stack of active menu ids (top = current).
    menu_stack: Vec<String>,
    /// Navigation input bindings.
    navigation: MenuNavigation,
    /// Current focus state for keyboard navigation.
    focus: MenuFocus,
    /// Action handler for custom menu actions.
    action_handler: Box<dyn MenuActionHandler>,
    /// The game viewport rect used to transform normalized menu coordinates to screen space.
    viewport: Rect,
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
    /// Overlay menu, game continues (except player movement).
    Overlay,
}

impl MenuMode {
    /// Returns true if the game logic should be pause.
    pub fn is_paused(&self) -> bool {
        matches!(self, MenuMode::Paused)
    }
}

impl MenuManager {
    /// Creates a new menu manager with default settings.
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            menu_stack: Vec::new(),
            navigation: MenuNavigation::default(),
            focus: MenuFocus::new(0),
            action_handler: Box::new(NoOpActionHandler),
            viewport: Rect::new(0.0, 0.0, 1.0, 1.0),
        };
        manager.register_default_menus();
        manager
    }

    /// Sets the game viewport rect so menu coordinates are correctly mapped to screen space.
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
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
        if let Some(template) = self.templates.get(id) {
            self.focus.reset(template);
            self.menu_stack.push(id.to_string());
        }
    }

    /// Closes the current menu and returns to previous menu if any.
    pub fn close_menu(&mut self) {
        self.menu_stack.pop();
        if let Some(parent_id) = self.menu_stack.last() {
            if let Some(template) = self.templates.get(parent_id) {
                self.focus.reset(template);
                return;
            }
        }
        self.focus = MenuFocus::new(0);
    }

    /// Closes all menus and returns to game.
    pub fn close_all(&mut self) {
        self.menu_stack.clear();
        self.focus = MenuFocus::new(0);
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

    /// Returns true if the bottom menu's background fully obscures the game.
    pub fn is_hiding_game(&self) -> bool {
        self.menu_stack.first()
            .and_then(|id| self.templates.get(id))
            .map_or(false, |t| t.background.is_opaque())
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
            if let Some(template) = self.templates.get(&menu_id).cloned() {
                if self.navigation.up_pressed(ctx) {
                    self.focus.navigate(NavDirection::Up, &template);
                }
                if self.navigation.down_pressed(ctx) {
                    self.focus.navigate(NavDirection::Down, &template);
                }
                if self.navigation.left_pressed(ctx) {
                    self.focus.navigate(NavDirection::Left, &template);
                }
                if self.navigation.right_pressed(ctx) {
                    self.focus.navigate(NavDirection::Right, &template);
                }

                let cancel_pressed = self.navigation.cancel_pressed(ctx);
                let confirm_pressed = self.navigation.confirm_pressed(ctx);
                let action_to_handle = if confirm_pressed {
                    template.get_element_at_focus(&self.focus)
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
    pub fn render<C: BishopContext>(&mut self, ctx: &mut C, text_manager: &TextManager) {
        if !self.has_active_menu() {
            return;
        }

        widgets_frame_start(ctx);

        let canvas_origin = Vec2::new(self.viewport.x, self.viewport.y);
        let canvas_size = Vec2::new(self.viewport.w, self.viewport.h);
        let mut triggered_action = None;

        if let Some(menu_id) = self.menu_stack.last() {
            let text_id = format!("ui/{}", menu_id);
            if let Some(template) = self.templates.get(menu_id) {
                template.render_background(ctx, self.viewport);
                template.render_labels(ctx, canvas_origin, canvas_size, text_manager, &text_id);

                for i in template.sorted_element_indices() {
                    let element = &template.elements[i];
                    if !element.visible {
                        continue;
                    }
                    match &element.kind {
                        MenuElementKind::Button(button) => {
                            let display_text = text_manager.resolve_ui_text(&text_id, &button.text_key);
                            let is_focused = self.focus.node == i && self.focus.child.is_none();
                            let screen_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                            let btn = Button::new(screen_rect, &display_text)
                                .blocked(!element.enabled)
                                .focused(is_focused);
                            if btn.show(ctx) {
                                triggered_action = Some(button.action.clone());
                            }
                        }
                        MenuElementKind::Panel(panel) => {
                            let screen_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                            ctx.draw_rectangle(
                                screen_rect.x,
                                screen_rect.y,
                                screen_rect.w,
                                screen_rect.h,
                                panel.background.render_color(),
                            );
                        }
                        MenuElementKind::LayoutGroup(group) => {
                            if let Some(bg) = &group.background {
                                let screen_rect = normalized_rect_to_screen(element.rect, canvas_origin, canvas_size);
                                ctx.draw_rectangle(
                                    screen_rect.x,
                                    screen_rect.y,
                                    screen_rect.w,
                                    screen_rect.h,
                                    bg.render_color(),
                                );
                            }
                            let resolved = resolve_layout(group, element.rect);
                            let mut focusable_idx = 0;
                            for (child, rect) in group.children.iter().zip(resolved.iter()) {
                                if !child.element.visible {
                                    continue;
                                }
                                if let MenuElementKind::Button(button) = &child.element.kind {
                                    let display_text = text_manager.resolve_ui_text(&text_id, &button.text_key);
                                    let is_focused = self.focus.node == i
                                        && self.focus.child == Some(focusable_idx);
                                    let screen_rect = normalized_rect_to_screen(*rect, canvas_origin, canvas_size);
                                    let btn = Button::new(screen_rect, &display_text)
                                        .blocked(!child.element.enabled)
                                        .focused(is_focused);
                                    if btn.show(ctx) {
                                        triggered_action = Some(button.action.clone());
                                    }
                                    if child.element.enabled {
                                        focusable_idx += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
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
        let layout = LayoutConfig::vertical()
            .with_item_size(200.0, 40.0)
            .with_spacing(16.0)
            .with_padding(Padding::uniform(32.0))
            .with_alignment(Alignment::center());

        let pause_menu = MenuBuilder::new("pause")
            .background(MenuBackground::Dimmed(0.7))
            .layout_group(
                Rect::new(0.0, 0.0, 1.0, 1.0),
                layout,
                |group| {
                    group
                        .label("Paused")
                        .button("Resume", MenuAction::Resume)
                },
            )
            .build();

        self.register_template(pause_menu);
    }

    /// Closes any active menu and resumes the game.
    pub fn close(&mut self) {
        self.close_all();
    }

    /// Loads all .ron menu templates from the menus folder and registers them.
    pub fn load_templates_from_disk(&mut self) {
        let dir = menus_folder();
        if !dir.exists() {
            return;
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            return;
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(true, |ext| ext != "ron") {
                continue;
            }

            let ron_str = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    onscreen_error!("Failed to read menu file {:?}: {}", path, e);
                    continue;
                }
            };

            match ron::de::from_str::<MenuTemplate>(&ron_str) {
                Ok(template) => self.register_template(template),
                Err(e) => onscreen_error!("Failed to parse menu file {:?}: {}", path, e),
            }
        }
    }
}
