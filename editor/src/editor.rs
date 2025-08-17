
use core::world::{world::World};

use crate::{room::room_editor::RoomEditor, world::world_editor::WorldEditor};

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
    pub fn new() -> Self {
        let world = World::new();
        Self {
            world,
            mode: EditorMode::World,
            world_editor: WorldEditor::new(),
            room_editor: RoomEditor::new(),
        }
    }

    pub fn update(&mut self) {
        match self.mode {
            EditorMode::World => {
                // Update returns the id of the room being edited
                if let Some(room_idx) = self.world_editor.update(&mut self.world) {
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
    }

    pub fn draw(&mut self) {
        match self.mode {
            EditorMode::World => {
                self.world_editor.draw(&self.world.rooms_metadata);
            }
            EditorMode::Room(room_idx) => {
                let room_metadata = &self.world.rooms_metadata[room_idx];
                self.room_editor.draw(room_metadata);
            }
        }
    }
}