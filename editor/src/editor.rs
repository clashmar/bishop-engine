use crate::controls::controls::Controls;
use crate::{storage::world_storage, room::room_editor::RoomEditor, world::world_editor::WorldEditor};
use core::world::{world::World};

pub enum EditorMode {
    World,
    Room(usize),
}

pub struct Editor {
    pub world: World,
    pub mode: EditorMode,
    pub world_editor: WorldEditor,
    pub room_editor: RoomEditor,
}

impl Editor {
    pub async fn new() -> Self {
        let world = if let Some(latest) = world_storage::most_recent_world() {
             world_storage::load_world(&latest)
        } else if let Some(name) = world_storage::prompt_user().await {
            world_storage::create_new_world(name)
        } else {
            // User pressed Escape -> fallback
            world_storage::create_new_world("untitled".to_string())
        };

        Self {
            world,
            mode: EditorMode::World,
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
        }
    }

    pub async fn update(&mut self) {
        match self.mode {
            EditorMode::World => {
                // Update returns the id of the room being edited
                if let Some(room_idx) = self.world_editor.update(&mut self.world).await {
                    self.mode = EditorMode::Room(room_idx);
                }
            }
            EditorMode::Room(room_idx) => {
                let rooms_metadata = &mut self.world.rooms_metadata;
                if self.room_editor.update(room_idx, rooms_metadata) {
                    self.world_editor.center_on_room(&rooms_metadata[room_idx]);
                    self.mode = EditorMode::World;
                }
            }
        }

        if Controls::save() {
            world_storage::save_world(&self.world).await;
        }
    }

    pub fn draw(&mut self) {
        match self.mode {
            EditorMode::World => {
                self.world_editor.draw(&self.world);
            }
            EditorMode::Room(room_idx) => {
                let room_metadata = &self.world.rooms_metadata[room_idx];
                self.room_editor.draw(room_metadata);
            }
        }
    }
}