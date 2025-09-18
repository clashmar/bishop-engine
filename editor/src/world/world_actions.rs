// editor/src/world/world_actions.rs
use engine_core::{constants::TILE_SIZE, tiles::tilemap::TileMap};
use uuid::Uuid;
use engine_core::{world::{room::{Room, RoomVariant}, world::World}};
use crate::world::coord;
use macroquad::prelude::*;
use crate::world::world_editor::WorldEditor;

impl WorldEditor {
    /// Create a new room and return its Uuid.
    pub fn create_room(
        &mut self,
        world: &mut World,
        name: &str,
        position: Vec2,
        size: Vec2,
    ) -> Uuid {
        let new_id = {
            let tilemap = TileMap::new(size.x as usize, size.y as usize);

            let variant = RoomVariant {
                id: "default".to_string(),
                tilemap,
            };

            let room = Room {
                id: Uuid::new_v4(),
                name: name.to_string(),
                position,
                size,
                exits: vec![],
                adjacent_rooms: vec![],
                variants: vec![variant],
            };

            let id = room.id;
            
            let _camera = room.create_room_camera(&mut world.world_ecs);

            world.rooms.push(room);
            id
        };

        let len = world.rooms.len(); 

        // Split the vector into “old rooms” and “the new room”
        let (old_slice, new_slice) = world.rooms.split_at_mut(len - 1);
        let new_room = &mut new_slice[0];

        for old_room in old_slice.iter_mut() {
            if Self::are_rooms_adjacent(old_room, new_room) {
                old_room.adjacent_rooms.push(new_id);
                new_room.adjacent_rooms.push(old_room.id);
            }
        }

        new_id
    }

    /// Delete a room by its UUID.
    pub fn delete_room(&mut self, world: &mut World, room_id: Uuid) {
        // Find the index of the room we want to remove
        let idx = match world.rooms.iter().position(|m| m.id == room_id) {
            Some(i) => i,
            None => return, // nothing to delete
        };

        // Remove the room from the world
        world.rooms.remove(idx);

        // Re‑compute adjacency for the remaining rooms
        let len = world.rooms.len();
        for i in 0..len {
            let (before, rest) = world.rooms.split_at_mut(i);
            let (room_i, after) = rest.split_first_mut().unwrap();
            room_i.adjacent_rooms.clear();

            for other in before.iter() {
                if Self::are_rooms_adjacent(room_i, other) {
                    room_i.adjacent_rooms.push(other.id);
                }
            }
            for other in after.iter() {
                if Self::are_rooms_adjacent(room_i, other) {
                    room_i.adjacent_rooms.push(other.id);
                }
            }
        }
    }

    /// Helper used by the UI when the user finishes a drag‑to‑place.
    pub fn place_room_from_drag(
        &mut self,
        world: &mut World,
        top_left: Vec2,
        size: Vec2,
    ) -> Uuid {
        let origin_in_pixels = top_left * TILE_SIZE;

        // The name could be generated automatically or asked from the UI.
        let new_id = self.create_room(world, "untitled", origin_in_pixels, size);

        new_id
    }

    fn are_rooms_adjacent(a: &Room, b: &Room) -> bool {
        let a_rect = Rect::new(a.position.x, a.position.y, a.size.x, a.size.y);
        let b_rect = Rect::new(b.position.x, b.position.y, b.size.x, b.size.y);

        // Rooms are adjacent if they share an edge
        let horizontal_touch = a_rect.x < b_rect.x + b_rect.w && a_rect.x + a_rect.w > b_rect.x &&
                            (a_rect.y + a_rect.h == b_rect.y || b_rect.y + b_rect.h == a_rect.y);
        let vertical_touch = a_rect.y < b_rect.y + b_rect.h && a_rect.y + a_rect.h > b_rect.y &&
                            (a_rect.x + a_rect.w == b_rect.x || b_rect.x + b_rect.w == a_rect.x);

        horizontal_touch || vertical_touch
    }

    pub fn draw_coordinates(&self, camera: &Camera2D) {
        let world_grid = coord::mouse_world_grid(camera);

        let txt = format!("({:.0}, {:.0})", world_grid.x, world_grid.y);

        let margin = 10.0;
        let x = margin;
        let y = screen_height() - margin; // baseline is at the bottom
        draw_text(&txt, x, y, 20.0, BLACK);
    }
}

