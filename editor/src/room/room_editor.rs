use core::world::room::{Room, RoomMetadata};
use macroquad::prelude::*;
use crate::tilemap::tilemap_editor::TileMapEditor;

pub enum RoomEditorMode {
    Tilemap,
    Scene,
}

pub struct RoomEditor {
    room: Option<Room>,
    pub mode: RoomEditorMode,
    pub tilemap_editor: TileMapEditor,
}

impl RoomEditor {
    pub fn new() -> Self {
        Self {
            room: None,
            mode: RoomEditorMode::Tilemap,
            tilemap_editor: TileMapEditor::new(),
        }
    }

    /// Returns `true` if user wants to exit back to world view.
    pub fn update(&mut self, room_idx: usize, rooms_metadata: &mut [RoomMetadata]) -> bool {
        if self.room.is_none() {
            let room_metadata = &mut rooms_metadata[room_idx];
            let room = room_metadata.load_room();
            self.room = Some(room);
        }

        // Get mutable reference to the current room
        let room = self.room.as_mut().unwrap();

        match self.mode {
            RoomEditorMode::Tilemap => {
                // Collect bounds for all other rooms to check for intersections
                let other_bounds: Vec<(Vec2, Vec2)> = rooms_metadata.iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != room_idx)
                    .map(|(_, r)| {
                        (r.position, r.size)
                    })
                    .collect();

                let tilemap = &mut room.variants[0].tilemap;
                let room_metadata = &mut rooms_metadata[room_idx];
                self.tilemap_editor.update(tilemap, room_metadata, &other_bounds);
            }
            RoomEditorMode::Scene => {
                // Non-tilemap logic
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.tilemap_editor.reset();
            self.reset();
            return true;
        }

        if is_key_pressed(KeyCode::Tab) {
            self.mode = match self.mode {
                RoomEditorMode::Tilemap => RoomEditorMode::Scene,
                RoomEditorMode::Scene => RoomEditorMode::Tilemap,
            };
        }

        false
    }

    pub fn draw(&mut self, room_metadata: &RoomMetadata) {
        if self.room.is_none() {
            let room = room_metadata.load_room();
            self.room = Some(room);
        }

        // Get mutable reference to the current room
        let room = self.room.as_mut().unwrap();

        match self.mode {
            RoomEditorMode::Tilemap => {
                let tilemap = &room.variants[0].tilemap;
                let exits = &room_metadata.exits;
                self.tilemap_editor.draw(tilemap, exits);
            }
            RoomEditorMode::Scene => {
                draw_text("Non-tilemap mode active", 20.0, 20.0, 24.0, WHITE);
            }
        }
    }

    pub fn reset(&mut self) {
        self.room = None;
        self.mode = RoomEditorMode::Tilemap;
    }
}