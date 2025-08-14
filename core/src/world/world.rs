use crate::{tilemap::TileMap, world::room::{Room, RoomVariant}};
use macroquad::prelude::*;

pub struct World {
    pub rooms: Vec<Room>,
}

impl World {
    pub fn new() -> Self {
        Self { rooms: Vec::new() }
    }

    pub fn create_room(&mut self, name: &str, position: Vec2, size: Vec2) -> usize {
        let variant = RoomVariant {
            id: "default".to_string(),
            tilemap: TileMap::new(size.x as usize, size.y as usize),
        };

        let room = Room {
            name: name.to_string(),
            position,
            variants: vec![variant],
            exits: vec![],
            adjacent_rooms: vec![],
        };

        self.rooms.push(room);
        let idx = self.rooms.len() - 1;

        // Update adjacency
        for i in 0..self.rooms.len() - 1 {
            if Self::are_rooms_adjacent(&self.rooms[i], &self.rooms[idx]) {
                self.rooms[i].adjacent_rooms.push(idx);
                self.rooms[idx].adjacent_rooms.push(i);
            }
        }

        idx
    }

    pub fn delete_room(&mut self, index: usize) {
        // Remove the room
        self.rooms.remove(index);

        // Recompute adjacency for all remaining rooms
        for i in 0..self.rooms.len() {
            self.rooms[i].adjacent_rooms.clear();
            for j in 0..self.rooms.len() {
                if i != j && Self::are_rooms_adjacent(&self.rooms[i], &self.rooms[j]) {
                    self.rooms[i].adjacent_rooms.push(j);
                }
            }
        }
    }

    pub fn link_all_exits(&mut self) {
        let len = self.rooms.len();

        for i in 0..len {
            // Split the rooms vector into two disjoint mutable slices
            let (left, right) = self.rooms.split_at_mut(i);
            let (room, right) = right.split_first_mut().unwrap(); // room = &mut self.rooms[i]

            // Create a slice of immutable references to all other rooms
            let other_rooms: Vec<&Room> = left.iter().chain(right.iter()).collect();

            room.link_exits_slice(&other_rooms);
        }
    }
    
    fn are_rooms_adjacent(a: &Room, b: &Room) -> bool {
        let a_rect = Rect::new(a.position.x, a.position.y, a.size().x, a.size().y);
        let b_rect = Rect::new(b.position.x, b.position.y, b.size().x, b.size().y);

        // Rooms are adjacent if they share an edge
        let horizontal_touch = a_rect.x < b_rect.x + b_rect.w && a_rect.x + a_rect.w > b_rect.x &&
                            (a_rect.y + a_rect.h == b_rect.y || b_rect.y + b_rect.h == a_rect.y);
        let vertical_touch = a_rect.y < b_rect.y + b_rect.h && a_rect.y + a_rect.h > b_rect.y &&
                            (a_rect.x + a_rect.w == b_rect.x || b_rect.x + b_rect.w == a_rect.x);

        horizontal_touch || vertical_touch
    }
}