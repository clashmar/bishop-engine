// editor/src/editor.rs
use crate::editor_actions::PROMPT_RESULT;
use crate::gui::inspector::modal::Modal;
use engine_core::ui::toast::Toast;
use engine_core::ui::widgets::input_is_focused;
use engine_core::world::world::WorldId;
use engine_core::world::room::RoomId;
use engine_core::physics::collider_system;
use engine_core::constants::*;
use engine_core::rendering::render_system::RenderSystem;
use std::io;
use macroquad::prelude::*;
use engine_core::game::game::Game;
use crate::gui::menu_bar::MenuBar;
use crate::controls::controls::Controls;
use crate::playtest::room_playtest;
use crate::tilemap::tile_palette::TilePalette;
use crate::editor_camera_controller::EditorCameraController;
use crate::storage::editor_storage;
use crate::Camera2D;
use crate::room::room_editor::RoomEditor;
use crate::world::world_editor::WorldEditor;
use crate::game::game_editor::GameEditor;

pub enum EditorMode {
    Game,
    World(WorldId),
    Room(RoomId),
}

pub struct Editor {
    pub game: Game,
    pub mode: EditorMode,
    pub game_editor: GameEditor,
    pub world_editor: WorldEditor,
    pub room_editor: RoomEditor,
    pub camera: Camera2D,
    pub current_world_id: Option<WorldId>,
    pub current_room_id: Option<RoomId>,
    pub render_system: RenderSystem,
    pub menu_bar: MenuBar,
    pub modal: Modal,
    pub toast: Option<Toast>,
}

impl Editor {
    pub async fn new() -> io::Result<Self> {
        let mut editor = Editor::default();

        let game = if let Some(name) = editor_storage::most_recent_game_name() {
            editor_storage::load_game_by_name(&name).await?
        } else if let Some(name) = editor_storage::prompt_user_input().await {
            editor_storage::create_new_game(name).await
        } else {
            // User pressed Escape
            editor_storage::create_new_game("untitled".to_string()).await
        };

        let palette = match editor_storage::load_palette(&game.name.clone()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to load palette: {e}");
                // Fall back to a new palette
                TilePalette::new()
            }
        };

        // TODO set camera for game editor 
        editor.camera = EditorCameraController::camera_for_room(
            DEFAULT_ROOM_SIZE,
            DEFAULT_ROOM_POSITION,
        );

        editor.game = game;

        // Give the palette to the tilemap editor
        editor.room_editor.tilemap_editor.tilemap_panel.palette = palette;

        Ok(editor)
    }

    pub async fn update(&mut self) {
        if !self.room_editor.view_preview && !self.room_editor.is_mouse_over_ui() {
            EditorCameraController::update(&mut self.camera);
        }
        
        match self.mode {
            EditorMode::Game => {
                // Returns the id of the world that was clicked on or None
                if let Some(world_id) = self.game_editor.update(
                    &mut self.game
                ).await {
                    self.current_world_id = Some(world_id);
                    self.mode = EditorMode::World(world_id);
                }
            }
            EditorMode::World(world_id) => {
                // Returns the id of the room that was clicked on or None
                if let Some(room_id) = self.world_editor.update(
                    &mut self.camera, 
                    &mut self.game.get_world(world_id)
                ).await {
                    self.current_room_id = Some(room_id);
                    self.mode = EditorMode::Room(room_id);
                }

                // Handle escape
                if Controls::escape() && !input_is_focused() {
                    // TODO: Handle camera

                    // Clean up
                    self.current_world_id = None;
                    self.world_editor.reset();
                    self.mode = EditorMode::Game;

                    // Save everything
                    self.save();
                }
            }
            EditorMode::Room(room_id) => {
                {
                    let current_world = &mut self.game.worlds
                        .iter_mut()
                        .find(|w| w.id == self.game.current_world_id)
                        .expect("Current world id not present in game.");

                    let other_bounds: Vec<(Vec2, Vec2)> = current_world.rooms
                        .iter()
                        .filter(|r| r.id != room_id)
                        .map(|r| (r.position, r.size))
                        .collect();
                    
                    let room = current_world.rooms
                        .iter_mut()
                        .find(|r| r.id == room_id)
                        .expect("Could not find room in world.");

                    // Returns true if escaped
                    self.room_editor.update(
                        &mut self.camera, 
                        room,
                        &other_bounds,
                        &mut current_world.world_ecs,
                        &mut self.game.asset_manager,
                    ).await;

                    collider_system::update_colliders_from_sprites(
                        &mut current_world.world_ecs,
                        &mut self.game.asset_manager,
                    );

                    if Controls::escape() && !input_is_focused() {
                        let palette = &mut self.room_editor.tilemap_editor.tilemap_panel.palette;
                        editor_storage::save_palette(palette, &self.game.name)
                            .expect("Could not save tile palette");

                        // Find the room we just left for center_on_room
                        if let Some(room) = current_world.rooms.iter()
                            .find(|m| m.id == room_id) {
                            self.world_editor.center_on_room(&mut self.camera, room);
                        }

                        // Clean up
                        self.current_room_id = None;
                        self.room_editor.reset();
                        self.mode = EditorMode::World(current_world.id);

                        // Save everything
                        self.save();
                    }
                }

                // Launch play‑test if the play button was pressed
                if self.room_editor.request_play {
                    // Write the payload
                    if let Some(room_id) = self.current_room_id {
                        let room = self.get_room_from_id(&room_id);
                        let payload_path = room_playtest::write_playtest_payload(room, &self.game);

                        // Build the binary first
                        match room_playtest::build_playtest_binary().await {
                            Ok(exe_path) => {
                                // Launch the binary
                                if let Err(e) = std::process::Command::new(&exe_path)
                                    .arg(&payload_path)
                                    .spawn()
                                {
                                    eprintln!("Failed to launch play‑test: {e}");
                                }
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                        // Reset the request flag so we don’t spawn multiple processes (and really ruin everything)
                        self.room_editor.request_play = false;      
                    }
                }
            }
        }

        self.handle_user_input().await;
    }

    pub async fn draw(&mut self) {
        match self.mode {
            EditorMode::Game => {
                self.game_editor.draw(
                    &mut self.game
                );
            },
            EditorMode::World(world_id) => {
                // World id should already be set
                if self.current_world_id.is_none() {
                    self.current_world_id = Some(world_id);
                }

                self.world_editor.draw(
                    world_id,
                    &self.camera, 
                    &mut self.game,
                );
            },
            EditorMode::Room(room_id) => {
                // Room id should already be set
                if self.current_room_id.is_none() {
                    self.current_room_id = Some(room_id);
                }
                
                let world = &mut self.game.worlds
                    .iter_mut()
                    .find(|w| w.id == self.game.current_world_id)
                    .expect("Current world id not present in game.");

                let room = world.rooms
                    .iter_mut()
                    .find(|r| r.id == room_id)
                    .expect("Could not find room in world.");

                self.room_editor.draw(
                    &self.camera,
                    room,
                    &mut world.world_ecs,
                    &mut self.game.asset_manager,
                    &mut self.render_system,
                ).await;
            }
        }

        // Draw global UI here
        self.draw_ui().await;
    }

    async fn draw_ui(&mut self) {
        // Modal
        if self.modal.is_open() {
            let clicked_outside = self.modal.draw(&mut self.game.asset_manager);
            if clicked_outside {
                self.modal.close();
                PROMPT_RESULT.with(|c| *c.borrow_mut() = None);
            }
        }

        // Global menu options
        self.draw_menu_bar().await;

        // Draw toast notifications
        if let Some(toast) = &mut self.toast {
            toast.update();
            if !toast.active {
                self.toast = None;
            }
        }
    }
}