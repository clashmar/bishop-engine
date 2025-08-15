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
    pub fn update(&mut self, room_idx: usize, rooms: &mut [Room]) -> bool {
        match self.mode {
            RoomEditorMode::Tilemap => {

                // Collect bounds for all other rooms to check for intersections
                let other_bounds: Vec<(Vec2, Vec2)> = rooms.iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != room_idx)
                    .map(|(_, r)| {
                        let size = Vec2::new(r.variants[0].tilemap.width as f32, r.variants[0].tilemap.height as f32);
                        (r.position, size)
                    })
                    .collect();

                let tilemap = &mut rooms[room_idx].variants[0].tilemap;
                let exits = &mut rooms[room_idx].exits;
                let position = &mut rooms[room_idx].position;
                self.tilemap_editor.update(tilemap, exits, position, &other_bounds);
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
                let exits = &room.exits;
                self.tilemap_editor.draw(tilemap, exits);
            }
            RoomEditorMode::Scene => {
                draw_text("Non-tilemap mode active", 20.0, 20.0, 24.0, WHITE);
            }
        }
    }
}