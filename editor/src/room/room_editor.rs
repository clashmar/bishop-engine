use core::world::room::Room;

use macroquad::prelude::*;
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

    /// Returns `true` if user wants to exit back to world view.
    pub fn update(&mut self, room: &mut Room) -> bool {
        match self.mode {
            RoomEditorMode::Tilemap => {
                let tilemap = &mut room.variants[0].tilemap;
                self.tilemap_editor.update(tilemap);
            }
            RoomEditorMode::Scene => {
                // Non-tilemap logic
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.tilemap_editor.reset();
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

    pub fn draw(&self, room: &Room) {
        match self.mode {
            RoomEditorMode::Tilemap => {
                let tilemap = &room.variants[0].tilemap;
                self.tilemap_editor.draw(tilemap);
            }
            RoomEditorMode::Scene => {
                draw_text("Non-tilemap mode active", 20.0, 20.0, 24.0, WHITE);
            }
        }
    }
}