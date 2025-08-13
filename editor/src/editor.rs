
use crate::{room::room_editor::RoomEditor, world::world_editor::WorldEditor};

pub enum EditorMode {
    World,
    Room(usize),
}

pub struct Editor {
    pub mode: EditorMode,
    pub world_editor: WorldEditor,
    pub room_editor: RoomEditor,
}

impl Editor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            world_editor: WorldEditor::new(width, height),
            room_editor: RoomEditor::new(),
            mode: EditorMode::World,
        }
    }

    pub fn update(&mut self) {
        match self.mode {
            EditorMode::World => {
                if let Some(room_idx) = self.world_editor.update() {
                    self.mode = EditorMode::Room(room_idx);
                }
            }
            EditorMode::Room(room_idx) => {
                let room = &mut self.world_editor.world.rooms[room_idx];
                if self.room_editor.update(room) {
                    self.world_editor.center_on_room(room_idx);
                    self.mode = EditorMode::World;
                }
            }
        }
    }

    pub fn draw(&self) {
        match self.mode {
            EditorMode::World => {
                self.world_editor.draw();
            }
            EditorMode::Room(room_idx) => {
                let room = &self.world_editor.world.rooms[room_idx];
                self.room_editor.draw(room);
            }
        }
    }
}