// editor/src/editor_actions.rs
use crate::gui::panels::diagnostics_panel::DIAGNOSTICS_PANEL;
use crate::gui::panels::hierarchy_panel::HIERARCHY_PANEL;
use crate::commands::game_editor_commands::RenameGameCmd;
use crate::gui::panels::console_panel::CONSOLE_PANEL;
use crate::editor_actions::world::world::WorldId;
use crate::world::world_editor::WorldEditor;
use crate::game::game_editor::GameEditor;
use crate::room::room_editor::RoomEditor;
use crate::ui::widgets::input_is_focused;
use crate::storage::export::export_game;
use crate::ecs::transform::Transform;
use engine_core::ui::widgets::*;
use crate::gui::menu_bar::*;
use crate::editor_global::*;
use crate::editor::Editor;
use crate::gui::modal::*;
use crate::editor::*;
use crate::storage::editor_storage::*;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::controls::controls::Controls;
use engine_core::storage::path_utils::*;
use engine_core::ui::toast::Toast;
use engine_core::game::game::Game;
use engine_core::world::room::*;
use engine_core::ui::prompt::*;
use macroquad::prelude::*;
use std::cell::RefCell;
use engine_core::*;

impl Default for Editor {
    fn default() -> Self {
        Self {
            game: Game::default(),
            camera: Camera2D::default(),
            mode: EditorMode::Game,
            game_editor: GameEditor::new(),
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
            cur_world_id: None,
            cur_room_id: None,
            render_system: RenderSystem::new(),
            menu_bar: MenuBar::new(),
            modal: Modal::default(),
            toast: None,
            playtest_process: None,
        }
    }
}

impl Editor {
    /// Returns `Some(name)` when the user confirms, `None` on cancel.
    pub async fn prompt_new_game(&mut self) -> Option<String> {
        self.open_new_game_modal();

        // Wait until the user has responded
        loop {
            // Draws and handles result
            if let Some(modal_result) = self.handle_modal().await {
                if let ModalResult::String(name) = modal_result {
                    // Only close modal if a name is returned
                    self.modal.close();
                    return Some(name);
                }
            }

            // Guard against modal not being open for some reason
            if !self.modal.is_open() {
                return None;
            }

            // Toasts can be created by the prompt
            self.draw_toast();

            next_frame().await;
        }
    }

    pub async fn draw_menu_bar(&mut self) {
        let menu_title = match self.mode {
            EditorMode::Game => self.game.name.clone(),
            EditorMode::World(_) => self.game.current_world().name.clone(),
            EditorMode::Room(id) => self
                .game
                .current_world()
                .get_room(id)
                .map(|room| room.name.clone())
                .unwrap_or_else(|| "Room".to_string()),
        };

        if let Some(action) = self.menu_bar.draw(&menu_title, self.mode) {
            match action {
                MenuAction::Rename => {
                    self.open_rename_modal();
                }
                MenuAction::NewGame => {
                    // Save current
                    self.save();
                    self.open_new_game_modal();
                }
                MenuAction::Open => {
                    // Open a folder picker rooted at the absolute save folder
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        use rfd::FileDialog;
                        if let Some(path) = FileDialog::new()
                            .set_directory(absolute_save_root())
                            .pick_folder()
                        {
                            match ensure_inside_save_root(&path) {
                                Ok(_) => {
                                    // Only load if it's in the correct folder
                                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                        match load_game_by_name(name).await {
                                            Ok(game) => {
                                                self.reset(game);
                                                self.toast = Some(Toast::new(
                                                    &format!("Loaded '{}'", name),
                                                    2.5,
                                                ));
                                            }
                                            Err(e) => {
                                                onscreen_error!("Failed to load game: {e}");
                                                self.toast = Some(Toast::new(
                                                    "Could not load selected game.",
                                                    2.5,
                                                ));
                                            }
                                        }
                                    } else {
                                        self.toast =
                                            Some(Toast::new("Folder name could not be read.", 2.5));
                                    }
                                }
                                Err(err_msg) => {
                                    self.toast = Some(Toast::new(&err_msg, 3.0));
                                }
                            }
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.toast = Some(Toast::new("Folder picker unavailable in WASM", 2.5));
                    }
                }
                MenuAction::Save => self.save(),
                MenuAction::SaveAs => self.open_save_as_modal(),
                MenuAction::Undo => crate::editor_global::request_undo(),
                MenuAction::Redo => crate::editor_global::request_redo(),
                MenuAction::Export => match export_game(&self.game).await {
                    Ok(path) => {
                        self.toast =
                            Some(Toast::new(format!("Exported to: {}", path.display()), 2.5));
                    }
                    Err(e) => {
                        onscreen_error!("Export failed: {e}");
                    }
                },
                MenuAction::ChangeSaveRoot => match change_save_root_async().await {
                    Some(new_root) => {
                        self.toast = Some(Toast::new(
                            format!("Save root moved to: {}", new_root.display()),
                            2.5,
                        ));
                    }
                    None => {
                        self.toast = Some(Toast::new("Failed to update save root.", 2.0));
                    }
                },
                MenuAction::ViewHierarchyPanel => {
                    with_panel_manager(|panel_manager| {
                        panel_manager.toggle(HIERARCHY_PANEL);
                    });
                }
                MenuAction::ViewConsolePanel => {
                    with_panel_manager(|panel_manager| {
                        panel_manager.toggle(CONSOLE_PANEL);
                    });
                }
                MenuAction::ViewDiagnosticsPanel => {
                    with_panel_manager(|panel_manager| {
                        panel_manager.toggle(DIAGNOSTICS_PANEL);
                    });
                }
                MenuAction::WorldSettings => {
                    self.open_world_settings_modal();
                }
            }
        }
    }

    pub async fn handle_shortcuts(&mut self) {
        if Controls::save() {
            self.save();
        }

        if Controls::save_as() {
            self.open_save_as_modal();
        }

        if Controls::undo() {
            crate::editor_global::request_undo();
        }

        if Controls::redo() {
            crate::editor_global::request_redo();
        }

        if Controls::c() && !input_is_focused() {
            with_panel_manager(|pm| pm.toggle(CONSOLE_PANEL));
        }

        if Controls::f3() && !input_is_focused() {
            with_panel_manager(|pm| pm.toggle(DIAGNOSTICS_PANEL));
        }
    }

    pub fn save(&mut self) {
        if let Err(e) = save_game(&self.game) {
            onscreen_error!("Could not save game: {}.", e)
        } else {
            self.toast = Some(Toast::new("Saved", 2.5));
        }
    }

    pub fn get_room_from_id(&self, room_id: &RoomId) -> &Room {
        self.game
            .current_world()
            .rooms
            .iter()
            .find(|m| m.id == *room_id)
            .expect("Could not find room from id.")
    }

    fn open_new_game_modal(&mut self) {
        let prompt_message = "Enter game name:";
        let mut prompt = self.set_prompt_modal(prompt_message);

        let widgets: Vec<BoxedWidget> = vec![Box::new(move |_| {
            if let Some(result) = prompt.draw() {
                // Write the result to the static thread local
                NEW_GAME_PROMPT_RESULT.with(|c| *c.borrow_mut() = Some(result));
            }
        })];

        self.modal.open(widgets);
    }

    fn open_rename_modal(&mut self) {
        let prompt_message = match self.mode {
            EditorMode::Game => "Rename game: ",
            EditorMode::World(_) => "Rename world: ",
            EditorMode::Room(_) => "Rename room: ",
        };

        let mut prompt = self.set_prompt_modal(prompt_message);

        let widgets: Vec<BoxedWidget> = vec![Box::new(move |_| {
            if let Some(result) = prompt.draw() {
                // Write the result to the static thread local
                RENAME_PROMPT_RESULT.with(|c| *c.borrow_mut() = Some(result));
            }
        })];

        self.modal.open(widgets);
    }

    fn open_save_as_modal(&mut self) {
        let prompt_message = "Save as:";
        let mut prompt = self.set_prompt_modal(prompt_message);

        let widgets: Vec<BoxedWidget> = vec![Box::new(move |_| {
            if let Some(result) = prompt.draw() {
                // Write the result to the static thread local
                SAVE_AS_PROMPT_RESULT.with(|c| *c.borrow_mut() = Some(result));
            }
        })];

        self.modal.open(widgets);
    }

    fn set_prompt_modal(&mut self, prompt_message: &str) -> StringPromptWidget {
        self.modal = Modal::new(400.0, 180.0);
        StringPromptWidget::new(self.modal.rect, prompt_message)
    }

    fn open_world_settings_modal(&mut self) {
        self.modal = Modal::new(300.0, 150.0);
        let world = self.game.current_world();
        let world_id = world.id;
        let grid_size = world.grid_size;

        let mut prompt = WorldSettingsPrompt::new(
            world_id,
            self.modal.rect,
            WidgetId::default(),
            grid_size,
        );

        let widgets: Vec<BoxedWidget> = vec![Box::new(move |_| {
            if let Some(result) = prompt.draw() {
                WORLD_SETTINGS_RESULT.with(|c| *c.borrow_mut() = Some(result));
            }
        })];

        self.modal.open(widgets);
    }

    pub async fn handle_modal(&mut self) -> Option<ModalResult> {
        if self.modal.is_open() {
            // Outside‑click handling
            if self.modal.draw(&mut self.game.asset_manager) {
                // Clear any pending results
                NEW_GAME_PROMPT_RESULT.with(|c| *c.borrow_mut() = None);
                return Some(ModalResult::ClickedOutside);
            }

            // New game name prompt
            let new_game_prompt_opt = NEW_GAME_PROMPT_RESULT.with(|c| c.borrow_mut().take());

            if let Some(result) = new_game_prompt_opt {
                match result {
                    StringPromptResult::Confirmed(name) => {
                        // Validation
                        if name.trim().is_empty() {
                            self.toast = Some(Toast::new("Name cannot be empty", 2.0));
                            return None;
                        } else {
                            if self.duplicate_game_exists(&name) {
                                return None;
                            }
                            // Create the new game
                            let new_game = create_new_game(name.clone()).await;
                            self.reset(new_game);
                            self.modal.close();
                            return Some(ModalResult::String(name));
                        }
                    }
                    StringPromptResult::Cancelled => {
                        self.modal.close();
                        return None;
                    }
                }
            }

            // Rename game name prompt
            let rename_prompt_opt = RENAME_PROMPT_RESULT.with(|c| c.borrow_mut().take());

            if let Some(result) = rename_prompt_opt {
                match result {
                    StringPromptResult::Confirmed(name) => {
                        match self.mode {
                            EditorMode::Game => {
                                if self.duplicate_game_exists(&name) {
                                    return None;
                                }
                                push_command(Box::new(RenameGameCmd::new(
                                    name,
                                    self.game.name.clone(),
                                )))
                            }
                            EditorMode::World(_) => self.game.current_world_mut().name = name,
                            EditorMode::Room(id) => {
                                if let Some(room) = self.game.current_world_mut().get_room_mut(id) {
                                    room.name = name;
                                }
                            }
                        }
                        self.modal.close();
                    }
                    StringPromptResult::Cancelled => {
                        self.modal.close();
                        return None;
                    }
                }
            }

            // Save as prompt
            let save_as_prompt_opt = SAVE_AS_PROMPT_RESULT.with(|c| c.borrow_mut().take());

            if let Some(result) = save_as_prompt_opt {
                match result {
                    StringPromptResult::Confirmed(name) => {
                        if self.duplicate_game_exists(&name) {
                            return None;
                        }
                        match save_as(&mut self.game, &name) {
                            Ok(()) => self.save(),
                            Err(err) => {
                                self.toast =
                                    Some(Toast::new(&format!("Failed to save game: {err}"), 3.0));
                            }
                        }
                        self.modal.close();
                    }
                    StringPromptResult::Cancelled => {
                        self.modal.close();
                        return None;
                    }
                }
            }

            // World settings prompt
            let world_settings_opt = WORLD_SETTINGS_RESULT.with(|c| c.borrow_mut().take());

            if let Some(result) = world_settings_opt {
                if let Some(new_grid_size) = result.grid_size {
                    self.apply_grid_size_change(result.id, new_grid_size);
                }
                self.modal.close();
            }
        }
        None
    }

    pub fn draw_toast(&mut self) {
        if let Some(toast) = &mut self.toast {
            toast.update();
            if !toast.active {
                self.toast = None;
            }
        }
    }

    pub fn reset(&mut self, game: Game) {
        *self = Self {
            game: game,
            camera: std::mem::take(&mut self.camera),
            ..Self::default()
        };
    }

    /// Returns `true` and creates a toast notification if a duplicate game name exists.
    fn duplicate_game_exists(&mut self, name: &String) -> bool {
        let duplicate_exists = list_game_names().iter().any(|existing| existing == name);

        if duplicate_exists {
            self.toast = Some(Toast::new(&format!("\"{name}\" already exists."), 2.5));
        };

        duplicate_exists
    }

    /// Applies a grid size change to the specified world, scaling room and entity positions.
    fn apply_grid_size_change(
        &mut self,
        world_id: WorldId,
        new_grid_size: f32,
    ) {
        let world = self.game.get_world_mut(world_id);
        let old_grid_size = world.grid_size;

        if (new_grid_size - old_grid_size).abs() < 0.001 {
            return;
        }

        let scale_factor = new_grid_size / old_grid_size;
        world.grid_size = new_grid_size;

        // Scale room positions
        for room in &mut world.rooms {
            room.position *= scale_factor;
        }

        // Scale entity positions
        let pos_store = self.game.ecs.get_store_mut::<Transform>();
        for (_entity, transform) in &mut pos_store.data {
            transform.position *= scale_factor;
        }

        self.toast = Some(Toast::new(
            &format!("World grid size changed to {new_grid_size}"),
            2.5,
        ));
    }
}

thread_local! {
    pub static NEW_GAME_PROMPT_RESULT: RefCell<Option<StringPromptResult>> = RefCell::new(None);
}

thread_local! {
    pub static RENAME_PROMPT_RESULT: RefCell<Option<StringPromptResult>> = RefCell::new(None);
}

thread_local! {
    pub static SAVE_AS_PROMPT_RESULT: RefCell<Option<StringPromptResult>> = RefCell::new(None);
}

thread_local! {
    pub static WORLD_SETTINGS_RESULT: RefCell<Option<WorldSettingsResult>> = RefCell::new(None);
}
