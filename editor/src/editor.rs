// editor/src/editor.rs
use engine_core::{
    assets::asset_manager::AssetManager, constants::*, game::game::Game, global::set_global_tile_size, physics::collider_system, rendering::render_system::RenderSystem, world::room::Room
};
use std::io;
use macroquad::prelude::*;
use uuid::Uuid;
use crate::{
    editor_camera_controller::EditorCameraController,
    controls::controls::Controls,
    room::room_editor::RoomEditor,
    storage::editor_storage,
    tilemap::tile_palette::TilePalette,
    world::world_editor::WorldEditor,
    playtest::room_playtest,
};

pub enum EditorMode {
    World,
    Room(Uuid),
}

pub struct Editor {
    pub game: Game,
    pub mode: EditorMode,
    pub world_editor: WorldEditor,
    pub room_editor: RoomEditor,
    pub camera: Camera2D,
    pub current_room_id: Option<Uuid>,
    pub asset_manager: AssetManager,
    pub light_system: RenderSystem,
}

impl Editor {
    pub async fn new() -> io::Result<Self> {
        let mut game = if let Some(name) = editor_storage::most_recent_game_name() {
            editor_storage::load_game_by_name(&name)?
        } else if let Some(name) = editor_storage::prompt_user_input().await {
            editor_storage::create_new_game(name)
        } else {
            // User pressed Escape
            editor_storage::create_new_game("untitled".to_string())
        };

        // Set global tile size that the game scales to
        set_global_tile_size(game.tile_size);

        let camera = EditorCameraController::camera_for_room(
            DEFAULT_ROOM_SIZE,
            DEFAULT_ROOM_POSITION,
        );

        let world = game.current_world_mut();

        let mut asset_manager = AssetManager::new(&mut world.world_ecs).await;

        let mut palette = match editor_storage::load_palette(&game.name) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to load palette: {e}");
                // Fall back to a new palette
                TilePalette::new()
            }
        };

        // Re‑load all sprite textures that belong to the palette.
        palette.rebuild_runtime(&mut asset_manager).await;

        let mut editor = Self {
            game,
            mode: EditorMode::World,
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
            camera,
            current_room_id: None,
            asset_manager,
            light_system: RenderSystem::new(),
        };

        // Give the palette to the tilemap editor
        editor.room_editor.tilemap_editor.panel.palette = palette;

        Ok(editor)
    }

    pub async fn update(&mut self) {
        if !self.room_editor.view_preview && !self.room_editor.is_mouse_over_ui() {
            EditorCameraController::update(&mut self.camera);
        }
        
        match self.mode {
            EditorMode::World => {
                // Update returns the id of the room being edited
                if let Some(room_id) = self.world_editor.update(&mut self.camera, &mut self.game.current_world_mut()).await {
                    self.current_room_id = Some(room_id);
                    self.mode = EditorMode::Room(room_id);
                }
            }
            EditorMode::Room(room_id) => {
                let world = self.game.current_world_mut();

                let other_bounds: Vec<(Vec2, Vec2)> = world.rooms
                    .iter()
                    .filter(|r| r.id != room_id)
                    .map(|r| (r.position, r.size))
                    .collect();
                
                let room = world.rooms
                    .iter_mut()
                    .find(|r| r.id == room_id)
                    .expect("Could not find room in world.");

                let done = {
                    // Returns true if escaped
                    self.room_editor.update(
                        &mut self.camera, 
                        room,
                        &other_bounds,
                        &mut world.world_ecs,
                        &mut self.asset_manager,
                    ).await
                };

                collider_system::update_colliders_from_sprites(
                    &mut self.game.current_world_mut().world_ecs,
                    &mut self.asset_manager,
                );

                // Launch play‑test if the play button was pressed
                if self.room_editor.request_play {
                    let world = self.game.current_world();

                    // Write the payload
                    if let Some(room_id) = &self.current_room_id {
                        let room = self.get_room_from_id(room_id);
                        let payload_path = room_playtest::write_playtest_payload(room, &world);

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
                    }
                    // Reset the request flag so we don’t spawn multiple processes (and really ruin everything)
                    self.room_editor.request_play = false;              
                }

                if done {
                    // Save everything
                    editor_storage::save_game(&self.game)
                        .expect("Could not save game.");

                    let palette = &mut self.room_editor.tilemap_editor.panel.palette;
                    editor_storage::save_palette(palette, &self.game.name)
                        .expect("Could not save tile palette");

                    // Find the room we just left for center_on_room
                    if let Some(room) = self.game.current_world_mut().rooms.iter()
                        .find(|m| m.id == room_id) {
                        self.world_editor.center_on_room(&mut self.camera, room);
                    }

                    // Clean up the temporary cache
                    self.current_room_id = None;
                    self.room_editor.reset();
                    self.mode = EditorMode::World;
                }
            }
        }

        if Controls::save() {
            editor_storage::save_game(&self.game)
                .expect("Could not save game.");
        }

        if Controls::undo() {
            crate::global::request_undo();
        }

        if Controls::redo() {
            crate::global::request_redo();
        }
    }

    pub fn draw(&mut self) {
        match self.mode {
            EditorMode::World => {
                self.world_editor.draw(
                    &self.camera, 
                    &mut self.game,
                );
            }
            EditorMode::Room(room_id) => {
                // The room id should already be set
                if self.current_room_id.is_none() {
                    self.current_room_id = Some(room_id);
                }

                let world = self.game.current_world_mut();

                let room = world.rooms
                    .iter_mut()
                    .find(|r| r.id == room_id)
                    .expect("Could not find room in world.");

                self.room_editor.draw(
                    &self.camera,
                    room,
                    &mut world.world_ecs,
                    &mut self.asset_manager,
                    &mut self.light_system,
                );
            }
        }
    }

    fn get_room_from_id(&self, room_id: &Uuid) -> &Room {
        self.game
            .current_world().rooms
            .iter()
            .find(|m| m.id == *room_id)
            .expect("Could not find room from id.")
    }
}