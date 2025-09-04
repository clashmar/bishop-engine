use uuid::Uuid;
use macroquad::prelude::*;
use crate::camera_controller::CameraController;
use crate::controls::controls::Controls;
use crate::tilemap::tile_palette::TilePalette;
use crate::{storage::world_storage, room::room_editor::RoomEditor, world::world_editor::WorldEditor};
use core::assets::asset_manager::AssetManager;
use core::world::room::Room;
use core::world::{world::World};
use core::constants::*;
use std::io;

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
    pub assets: AssetManager,
}

impl Editor {
    pub async fn new() -> io::Result<Self> {
        let world = if let Some(latest_id) = world_storage::most_recent_world_id() {
             world_storage::load_world_by_id(&latest_id).expect("Could not load world")
        } else if let Some(name) = world_storage::prompt_user_input().await {
            world_storage::create_new_world(name)
        } else {
            // User pressed Escape
            world_storage::create_new_world("untitled".to_string())
        };

        let camera = CameraController::camera_for_room(
            DEFAULT_ROOM_SIZE,
            DEFAULT_ROOM_POSITION,
        );

        let mut assets = AssetManager::new();

        let mut palette = match world_storage::load_palette(&world.id) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to load palette: {e}");
                // Fall back to a new palette
                TilePalette::new(vec2(10.0, 10.0), 32.0, 2, 2)
            }
        };

        // Re‑load all sprite textures that belong to the palette.
        palette.rebuild_runtime(&mut assets).await;

        let mut editor = Self {
            world,
            mode: EditorMode::World,
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
            camera,
            current_room: None,
            current_room_id: None,
            assets,
        };

        // Give the palette to the tilemap editor
        editor.room_editor.tilemap_editor.palette = palette;
        editor.room_editor.entity_palette.load_prefabs_from_disk(&editor.world.id, &mut editor.assets);

        Ok(editor)
    }

    pub async fn update(&mut self) {
        CameraController::update(&mut self.camera);
        match self.mode {
            EditorMode::World => {
                // Update returns the id of the room being edited
                if let Some(room_id) = self.world_editor.update(&mut self.camera, &mut self.world).await {
                    match world_storage::load_room(&self.world.id, room_id) {
                        Ok(room) => {
                            self.current_room = Some(room);
                            self.current_room_id = Some(room_id);
                            self.sync_assets();
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
                            &mut self.assets,
                            &self.world.id,
                        )
                };

                if done {
                    // Take the edited room out of the editor
                    if let Some(ref edited_room) = self.current_room {
                        if let Err(e) = world_storage::save_room(
                            &self.world.id,
                            room_id,
                            edited_room,
                        ) {
                            eprintln!("Could not save room {room_id}: {e}");
                        }
                        world_storage::save_world(&self.world)
                            .expect("Could not save world.");

                        if let Some(_) = self.current_room_id {
                            let palette = &mut self.room_editor.tilemap_editor.palette;
                            world_storage::save_palette(palette, &self.world.id)
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
            world_storage::save_world(&self.world)
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
                    match world_storage::load_room(&self.world.id, room_id) {
                        Ok(room) => self.current_room = Some(room),
                        Err(e) => eprintln!("Failed to load room {room_id}: {e}"),
                    }
                }

                if let Some(ref room) = self.current_room {
                    self.room_editor.draw(
                        &self.camera, 
                        room, 
                        meta, 
                        &mut self.world.world_ecs,
                        &mut self.assets,
                    );
                }
            }
        }
    }

    fn sync_assets(&mut self) {
        // Iterate over all non-tile sprites
        for (_entity, sprite) in self.world.world_ecs.sprites.data.iter_mut() {
            if !self.assets.contains(sprite.sprite_id) {
                let id = futures::executor::block_on(self.assets.load(&sprite.path));
                sprite.sprite_id = id;
            }
        }

        // Iterate over all tile‑sprites
        for (_entity, tile_sprite) in self.world.world_ecs.tile_sprites.data.iter_mut() {
            if !self.assets.contains(tile_sprite.sprite_id) {
                let new_id = futures::executor::block_on(self.assets.load(&tile_sprite.path));
                tile_sprite.sprite_id = new_id;
            }
        }
    }
}