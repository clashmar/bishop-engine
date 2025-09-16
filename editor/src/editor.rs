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
    pub current_room: Option<Room>,
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
            current_room: None,
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
                    match editor_storage::load_room(&self.world.id, room_id) {
                        Ok(room) => {
                            self.current_room = Some(room);
                            self.current_room_id = Some(room_id);
                            self.mode = EditorMode::Room(room_id);
                        }
                        Err(e) => {
                            eprintln!("Failed to load room {room_id}: {e}");
                        }
                    }
                }
            }
            EditorMode::Room(room_id) => {
                let done = {
                    let meta_slice = &mut self.world.rooms_metadata[..];
                    let room = self.current_room.as_mut().expect("room must be loaded");

                    // Returns true if escaped
                    self.room_editor.update(
                            &mut self.camera, 
                            room, 
                            room_id, 
                            meta_slice,
                            &mut self.world.world_ecs,
                            &mut self.asset_manager,
                        ).await
                };

                // -------------------------------------------------------------
                // 3️⃣  Launch play‑test if the button was pressed
                // -------------------------------------------------------------
                if self.room_editor.request_play {
                    // The room is already loaded in `self.current_room`
                    if let (Some(room), Some(meta)) = (&self.current_room, self.world.rooms_metadata.iter()
                        .find(|m| m.id == room_id)) {

                        // 1️⃣  Serialize everything the play‑test binary needs
                        let payload_path = room_playtest::write_playtest_payload(room, meta, &self.world);

                        // 2️⃣  Spawn the play‑test binary as a child process.
                        //    The binary is the second binary defined in Cargo.toml:
                        //    `game-playtest`.
                        #[cfg(target_os = "windows")]
                        let exe_name = "game-playtest.exe";
                        #[cfg(not(target_os = "windows"))]
                        let exe_name = "game-playtest";

                        // Resolve the path relative to the workspace root.
                        // `env!("CARGO_MANIFEST_DIR")` points to `editor/`.
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
                    // Take the edited room out of the editor
                    if let Some(ref edited_room) = self.current_room {
                        if let Err(e) = editor_storage::save_room(
                            &self.world.id,
                            room_id,
                            edited_room,
                        ) {
                            eprintln!("Could not save room {room_id}: {e}");
                        }
                        editor_storage::save_world(&self.world)
                            .expect("Could not save world.");

                        if let Some(_) = self.current_room_id {
                            let palette = &mut self.room_editor.tilemap_editor.panel.palette;
                            editor_storage::save_palette(palette, &self.world.id)
                                .expect("Could not save tile palette");
                        }
                    }

                    // Find the metadata for the room we just left for center_on_room.
                    if let Some(meta) = self.world.rooms_metadata.iter()
                        .find(|m| m.id == room_id) {
                        self.world_editor.center_on_room(&mut self.camera, meta);
                    }

                    // Clean up the temporary cache.
                    self.current_room = None;
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
                let meta = self.world.rooms_metadata
                    .iter()
                    .find(|m| m.id == room_id)
                    .expect("metadata must exist");

                // The room should already be loaded but lazy loads if not
                if self.current_room.is_none() {
                    match editor_storage::load_room(&self.world.id, room_id) {
                        Ok(room) => self.current_room = Some(room),
                        Err(e) => eprintln!("Failed to load room {room_id}: {e}"),
                    }
                }

                if let Some(ref mut room) = &mut self.current_room {
                    self.room_editor.draw(
                        &self.camera, 
                        room, 
                        meta, 
                        &mut self.world.world_ecs,
                        &mut self.asset_manager,
                    );
                }
            }
        }
    }
}