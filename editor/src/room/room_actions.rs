// editor/src/room/room_actions.rs
use engine_core::world::room::Room;
use crate::room::room_editor::RoomEditor;
use macroquad::prelude::*;
use crate::world::coord;

impl RoomEditor {
    /// Draw the cursor coordinates in world space.
    pub fn draw_coordinates(&self, camera: &Camera2D, room: &Room) {
        let local_grid = coord::mouse_world_grid(camera);

        let world_grid = local_grid + room.position;
        
        let txt = format!(
            "({:.0}, {:.0})",
            world_grid.x, world_grid.y,
        );

        let margin = 10.0;
        draw_text(&txt, margin, screen_height() - margin, 20.0, BLUE);
    }
}