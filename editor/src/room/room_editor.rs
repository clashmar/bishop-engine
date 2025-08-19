use core::world::room::{Room, RoomMetadata};
use macroquad::prelude::*;
use uuid::Uuid;
use crate::tilemap::tilemap_editor::TileMapEditor;

pub enum RoomEditorMode {
    Tilemap,
    Scene,
}

pub struct RoomEditor {
    pub mode: RoomEditorMode,
    pub tilemap_editor: TileMapEditor,
}

impl RoomEditor {
    pub fn new() -> Self {
        Self {
            mode: RoomEditorMode::Tilemap,
            tilemap_editor: TileMapEditor::new(),
        }
    }

    /// Returns `true` if user wants to exit back to world view. // Still refactoring as I go 
    pub fn update(
        &mut self, 
        room: &mut Room,
        room_id: Uuid, 
        rooms_metadata: &mut [RoomMetadata]
    ) -> bool {
        match self.mode {
            RoomEditorMode::Tilemap => {
                // Collect bounds for all other rooms to check for intersections
                let other_bounds: Vec<(Vec2, Vec2)> = rooms_metadata
                    .iter()
                    .filter(|m| m.id != room_id)
                    .map(|m| (m.position, m.size))
                    .collect();

                let tilemap = &mut room.variants[0].tilemap;

                let room_metadata = rooms_metadata
                    .iter_mut()
                    .find(|m| m.id == room_id)
                    .expect("metadata must still exist");

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

    pub fn draw(
        &mut self, 
        room: &Room,
        room_metadata: &RoomMetadata
    ) {
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
        self.mode = RoomEditorMode::Tilemap;
    }
}