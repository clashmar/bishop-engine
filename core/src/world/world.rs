use serde_with::FromInto;
use crate::{world::room::{RoomMetadata}};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct World {
    pub name: String,
    pub rooms_metadata: Vec<RoomMetadata>,
    pub starting_room: Option<usize>,
    #[serde_as(as = "Option<FromInto<[f32; 2]>>")]
    pub starting_position: Option<Vec2>,
}

impl World {
    pub fn create_room(&mut self, name: &str, position: Vec2, size: Vec2) -> usize {

        let metadata = RoomMetadata {
            name: name.to_string(),
            position,
            size,
            exits: vec![],
            adjacent_rooms: vec![],
        };

        self.rooms_metadata.push(metadata);
        let idx = self.rooms_metadata.len() - 1;

        // Update adjacency
        for i in 0..self.rooms_metadata.len() - 1 {
            if Self::are_rooms_adjacent(&self.rooms_metadata[i], &self.rooms_metadata[idx]) {
                self.rooms_metadata[i].adjacent_rooms.push(idx);
                self.rooms_metadata[idx].adjacent_rooms.push(i);
            }
        }

        idx
    }

    pub fn delete_room(&mut self, index: usize) {
        // Remove the room
        self.rooms_metadata.remove(index);

        // Recompute adjacency for all remaining rooms
        for i in 0..self.rooms_metadata.len() {
            self.rooms_metadata[i].adjacent_rooms.clear();
            for j in 0..self.rooms_metadata.len() {
                if i != j && Self::are_rooms_adjacent(&self.rooms_metadata[i], &self.rooms_metadata[j]) {
                    self.rooms_metadata[i].adjacent_rooms.push(j);
                }
            }
        }
    }

    pub fn link_all_exits(&mut self) {
        let len = self.rooms_metadata.len();

        for i in 0..len {
            // Split the room metadata vector into two disjoint mutable slices
            let (left, right) = self.rooms_metadata.split_at_mut(i);
            let (room_metadata, right) = right.split_first_mut().unwrap(); // room = &mut self.rooms[i]

            // Create a slice of immutable references to all other rooms
            let other_rooms: Vec<&RoomMetadata> = left.iter().chain(right.iter()).collect();

            room_metadata.link_exits(&other_rooms);
        }
    }
    
    fn are_rooms_adjacent(a: &RoomMetadata, b: &RoomMetadata) -> bool {
        let a_rect = Rect::new(a.position.x, a.position.y, a.size.x, a.size.y);
        let b_rect = Rect::new(b.position.x, b.position.y, b.size.x, b.size.y);

        // Rooms are adjacent if they share an edge
        let horizontal_touch = a_rect.x < b_rect.x + b_rect.w && a_rect.x + a_rect.w > b_rect.x &&
                            (a_rect.y + a_rect.h == b_rect.y || b_rect.y + b_rect.h == a_rect.y);
        let vertical_touch = a_rect.y < b_rect.y + b_rect.h && a_rect.y + a_rect.h > b_rect.y &&
                            (a_rect.x + a_rect.w == b_rect.x || b_rect.x + b_rect.w == a_rect.x);

        horizontal_touch || vertical_touch
    }
}