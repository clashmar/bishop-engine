// editor/src/editor.rs
use std::io;
use async_std::path::PathBuf;
use macroquad::prelude::*;
use uuid::Uuid;
use crate::{
    camera_controller::CameraController,
    controls::controls::Controls,
    room::room_editor::RoomEditor,
    storage::editor_storage,
    tilemap::tile_palette::TilePalette,
    world::world_editor::WorldEditor,
    playtest::room_playtest,
};
use engine_core::{
    assets::
        asset_manager::AssetManager
    , 
    constants::*, 
    world::{
        room::Room,
        world::World,
    },
    storage::core_storage,
};

pub enum EditorMode {
    World,
    Room(Uuid),
}

pub struct Editor {
    pub world: World,
    pub mode: EditorMode,
    pub world_editor: WorldEditor,
    pub room_editor: RoomEditor,
    pub camera: Camera2D,
    pub current_room_id: Option<Uuid>,
    pub asset_manager: AssetManager,
}

impl Editor {
    pub async fn new() -> io::Result<Self> {
        let mut world = if let Some(latest_id) = editor_storage::most_recent_world_id() {
            core_storage::load_world_by_id(&latest_id).expect("Could not load world")
        } else if let Some(name) = editor_storage::prompt_user_input().await {
            editor_storage::create_new_world(name)
        } else {
            // User pressed Escape
            editor_storage::create_new_world("untitled".to_string())
        };

        let camera = CameraController::camera_for_room(
            DEFAULT_ROOM_SIZE,
            DEFAULT_ROOM_POSITION,
        );

        let mut asset_manager = AssetManager::new(&mut world.world_ecs).await;

        let mut palette = match editor_storage::load_palette(&world.id) {
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
            world,
            mode: EditorMode::World,
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
            camera,
            current_room_id: None,
            asset_manager,
        };

        // Give the palette to the tilemap editor
        editor.room_editor.tilemap_editor.panel.palette = palette;

        Ok(editor)
    }

    pub async fn update(&mut self) {
        CameraController::update(&mut self.camera);
        match self.mode {
            EditorMode::World => {
                // Update returns the id of the room being edited
                if let Some(room_id) = self.world_editor.update(&mut self.camera, &mut self.world).await {
                    self.current_room_id = Some(room_id);
                    self.mode = EditorMode::Room(room_id);
                }
            }
            EditorMode::Room(room_id) => {
                let other_bounds: Vec<(Vec2, Vec2)> = self.world.rooms
                    .iter()
                    .filter(|r| r.id != room_id)
                    .map(|r| (r.position, r.size))
                    .collect();
                
                let room = self.world.rooms
                    .iter_mut()
                    .find(|r| r.id == room_id)
                    .expect("Could not find room in world.");

                let done = {
                    // Returns true if escaped
                    self.room_editor.update(
                        &mut self.camera, 
                        room,
                        &other_bounds,
                        &mut self.world.world_ecs,
                        &mut self.asset_manager,
                    ).await
                };

                // Launch play‑test if the play button was pressed
                if self.room_editor.request_play {
                    if let Some(room_id) = &self.current_room_id {
                        let room = self.get_room_from_id(room_id);

                        // Serialize everything the play‑test binary needs
                        let payload_path = room_playtest::write_playtest_payload(room, &self.world);

                        // Spawn the play‑test binary as a child process
                        #[cfg(target_os = "windows")]
                        let exe_name = "game-playtest.exe";
                        #[cfg(not(target_os = "windows"))]
                        let exe_name = "game-playtest";

                        // Resolve the path relative to the workspace root
                        let mut exe_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                        exe_path.pop(); // go up to workspace root
                        exe_path.push("target");
                        exe_path.push("debug");
                        exe_path.push(exe_name);

                        // Launch – we deliberately ignore the child’s stdout/stderr; the
                        // editor continues running.
                        if let Err(e) = std::process::Command::new(exe_path)
                            .arg(&payload_path)
                            .spawn()
                        {
                            eprintln!("Failed to launch play‑test: {e}");
                        }
                    }

                    // Prevent a million games being spawned and ruining everything
                    self.room_editor.request_play = false; 
                }

                if done {
                    // Save everything
                    editor_storage::save_world(&self.world)
                        .expect("Could not save world.");

                    let palette = &mut self.room_editor.tilemap_editor.panel.palette;
                    editor_storage::save_palette(palette, &self.world.id)
                        .expect("Could not save tile palette");

                    // Find the room we just left for center_on_room
                    if let Some(room) = self.world.rooms.iter()
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
            editor_storage::save_world(&self.world)
                .expect("Could not save world.");
        }
    }

    pub fn draw(&mut self) {
        match self.mode {
            EditorMode::World => {
                self.world_editor.draw(&self.camera, &self.world);
            }
            EditorMode::Room(room_id) => {
                // The room id should already be set
                if self.current_room_id.is_none() {
                    self.current_room_id = Some(room_id);
                }

                let room = self.world.rooms
                    .iter_mut()
                    .find(|r| r.id == room_id)
                    .expect("Could not find room in world.");

                self.room_editor.draw(
                    &self.camera,
                    room,
                    &mut self.world.world_ecs,
                    &mut self.asset_manager,
                );
            }
        }
    }

    fn get_room_from_id(&self, room_id: &Uuid) -> &Room {
        self.world.rooms.iter().find(|m| m.id == *room_id).expect("Could not find room from id.")
    }
}

