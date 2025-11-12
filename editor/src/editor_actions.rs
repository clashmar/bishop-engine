// editor/src/editor_actions.rs
use crate::controls::controls::Controls;
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
use crate::gui::inspector::modal::Modal;
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
                MenuAction::NewGame => {
                    // Save current
                    self.save();

                    let prompt_message = "Enter game name:";
                    self.modal = Modal::new(400.0, 180.0);
                    let mut prompt = StringPromptWidget::new(self.modal.rect, prompt_message);

                    self.modal.open(move |_asset_manager| {
                        if let Some(result) = prompt.draw() {
                            // Write the result to the static thread local
                            PROMPT_RESULT.with(|c| *c.borrow_mut() = Some(result));
                        }
                    });
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
        // Modal
        if self.modal.is_open() {
            let prompt_result_opt = PROMPT_RESULT.with(|c| c.borrow_mut().take());

            if let Some(result) = prompt_result_opt {
                self.modal.close();

                match result {
                    StringPromptResult::Confirmed(name) => {
                        // Validation
                        if name.trim().is_empty() {
                            self.toast = Some(Toast::new("Name cannot be empty", 2.0));
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
                            } else {
                                // Create the new game
                                let new_game = editor_storage::create_new_game(name).await;
                                self.reset(new_game);
                            }
                        }
                    }
                    StringPromptResult::Cancelled => { }
                }
            }

            // Outside‑click handling
            if self.modal.draw(&mut self.game.asset_manager) {
                self.modal.close();
                // Clear any pending result
                PROMPT_RESULT.with(|c| *c.borrow_mut() = None);
            }
        }

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

    pub fn reset(&mut self, game: Game) {
        *self = Self {
            game: game,
            camera: std::mem::take(&mut self.camera), // TODO: set camera correctly
            ..Self::default()
        };
    }
}

thread_local! {
    pub static PROMPT_RESULT: RefCell<Option<StringPromptResult>> = RefCell::new(None);
}