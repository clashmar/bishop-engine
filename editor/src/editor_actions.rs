// editor/src/editor_actions.rs
use engine_core::controls::controls::Controls;
use engine_core::ui::prompt::StringPromptResult;
use std::cell::RefCell;
use engine_core::ui::prompt::StringPromptWidget;
use engine_core::world::room::*;
use macroquad::prelude::*;
use engine_core::game::game::Game;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::storage::path_utils::*;
use engine_core::ui::toast::Toast;
use crate::game::game_editor::GameEditor;
use crate::gui::inspector::modal::*;
use crate::room::room_editor::RoomEditor;
use crate::world::world_editor::WorldEditor;
use crate::editor::*;
use crate::storage::editor_storage;
use crate::gui::menu_bar::*;
use crate::editor::Editor;

impl Default for Editor {
    fn default() -> Self {
        Self {
            game: Game::default(),
            camera: Camera2D::default(),
            mode: EditorMode::Game,
            game_editor: GameEditor::new(),
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
            current_world_id: None,
            current_room_id: None,
            render_system: RenderSystem::new(),
            menu_bar: MenuBar::new(),
            modal: Modal::default(),
            toast: None,
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
            EditorMode::Game => {
                &self.game.name
            }
            EditorMode::World(_) => {
                &self.game.current_world().name
            }
            EditorMode::Room(_) => {
                &self.current_room_id
                    .and_then(|room_id| {
                        Some(self.get_room_from_id(&room_id).name.clone())
                    })
                    .unwrap_or_else(|| "Room".to_string())
            }
        };

        if let Some(action) = self.menu_bar.draw(&menu_title) {
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
                                        match editor_storage::load_game_by_name(name).await {
                                            Ok(game) => {
                                                self.reset(game);
                                                self.toast = Some(Toast::new(
                                                    &format!("Loaded '{}'", name),
                                                    2.5,
                                                ));
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to load game: {e}");
                                                self.toast = Some(Toast::new(
                                                    "Could not load selected game.",
                                                    2.5,
                                                ));
                                            }
                                        }
                                    } else {
                                        self.toast = Some(Toast::new(
                                            "Folder name could not be read.",
                                            2.5,
                                        ));
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
                        self.toast = Some(Toast::new(
                            "Folder picker unavailable in WASM",
                            2.5,
                        ));
                    }
                }
                MenuAction::Save => self.save(),
                _ => {}
            }
        }
    }

    pub async fn handle_user_input(&mut self) {
        if Controls::save() {
            self.save();
        }

        if Controls::undo() {
            crate::global::request_undo();
        }

        if Controls::redo() {
            crate::global::request_redo();
        }
    }

    pub fn save(&mut self) {
        editor_storage::save_game(&self.game)
                .expect("Could not save game.");
        self.toast = Some(Toast::new("Saved", 2.5));
    }

    pub fn get_room_from_id(&self, room_id: &RoomId) -> &Room {
        self.game
            .current_world().rooms
            .iter()
            .find(|m| m.id == *room_id)
            .expect("Could not find room from id.")
    }

    fn open_new_game_modal(&mut self) {
        let prompt_message = "Enter game name:";
        self.modal = Modal::new(400.0, 180.0);
        let mut prompt = StringPromptWidget::new(self.modal.rect, prompt_message);

        let widgets: Vec<BoxedWidget> = vec![ 
            Box::new(move |_| {
                if let Some(result) = prompt.draw() {
                    // Write the result to the static thread local
                    NEW_GAME_PROMPT_RESULT.with(|c| *c.borrow_mut() = Some(result));
                }
            })
        ];

        self.modal.open(widgets);
    }

    fn open_rename_modal(&mut self) {
        let prompt_message = match self.mode {
            EditorMode::Game => "Rename game: ",
            EditorMode::World(_) => "Rename world: ",
            EditorMode::Room(_) => "Rename room: ",
        };

        self.modal = Modal::new(400.0, 180.0);
        let mut prompt = StringPromptWidget::new(self.modal.rect, prompt_message);

        let widgets: Vec<BoxedWidget> = vec![ 
            Box::new(move |_| {
                if let Some(result) = prompt.draw() {
                    // Write the result to the static thread local
                    RENAME_PROMPT_RESULT.with(|c| *c.borrow_mut() = Some(result));
                }
            })
        ];

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
                            // Duplicate check (case‑sensitive)
                            let duplicate = editor_storage::list_game_names()
                                .iter()
                                .any(|existing| existing == &name);

                            if duplicate {
                                self.toast = Some(Toast::new(
                                    &format!("\"{name}\" already exists."),
                                    2.5,
                                ));
                                return None;
                            } else {
                                // Create the new game
                                let new_game = editor_storage::create_new_game(name.clone()).await;
                                self.reset(new_game);
                                self.modal.close();
                                return Some(ModalResult::String(name));
                            }
                        }
                    }
                    StringPromptResult::Cancelled => { 
                        self.modal.close();
                        return None
                    }
                }
            }

            // New game name prompt
            let rename_prompt_opt = RENAME_PROMPT_RESULT.with(|c| c.borrow_mut().take());

            if let Some(result) = rename_prompt_opt {
                match result {
                    StringPromptResult::Confirmed(name) => {
                        match self.mode {
                            EditorMode::Game => {
                                self.game.name = name
                            } 
                            EditorMode::World(_) => {
                                self.game.current_world_mut().name = name
                            },
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
                        return None
                    }
                }
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
            camera: std::mem::take(&mut self.camera), // TODO: set camera correctly
            ..Self::default()
        };
    }
}

thread_local! {
    pub static NEW_GAME_PROMPT_RESULT: RefCell<Option<StringPromptResult>> = RefCell::new(None);
}

thread_local! {
    pub static RENAME_PROMPT_RESULT: RefCell<Option<StringPromptResult>> = RefCell::new(None);
}