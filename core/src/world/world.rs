use serde_with::FromInto;
use uuid::Uuid;
use crate::{world::room::{RoomMetadata}};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct World {
    pub name: String,
    pub rooms_metadata: Vec<RoomMetadata>,
    pub starting_room: Option<Uuid>,
    #[serde_as(as = "Option<FromInto<[f32; 2]>>")]
    pub starting_position: Option<Vec2>,
}

impl World {
    /// Create a new room and return its Uuid.
    // pub fn create_room(&mut self, name: &str, position: Vec2, size: Vec2) -> Uuid {
    //     let new_meta = RoomMetadata {
    //         id: Uuid::new_v4(),
    //         name: name.to_string(),
    //         position,
    //         size,
    //         exits: vec![],
    //         adjacent_rooms: vec![],
    //     };

    //     let new_id = new_meta.id;
    //     self.rooms_metadata.push(new_meta);

    //     // Split the vector into “old rooms” and “the new room”
    //     let len = self.rooms_metadata.len();              
    //     let (old_slice, new_slice) = self.rooms_metadata.split_at_mut(len - 1);

    //     // The new room
    //     let new_meta_ref = &mut new_slice[0];

    //     // Iterate through old rooms and update adjacency
    //     for old_meta in old_slice.iter_mut() {
    //         if Self::are_rooms_adjacent(old_meta, new_meta_ref) {
    //             // Each room stores the other's UUID
    //             old_meta.adjacent_rooms.push(new_id);
    //             new_meta_ref.adjacent_rooms.push(old_meta.id);
    //         }
    //     }

    //     // Save the room.
    //     if let Err(e) = world_storage::save_room(
    //         &world.name,             
    //         room_id,             
    //         &first_room,        
    //     ) {
    //         eprintln!("Could not save the initial room: {e}");
    //     }

    //     new_id
    // }

    // /// Delete a room by its UUID.
    // pub fn delete_room(&mut self, room_id: Uuid) {
    //     // Find the index of the room we want to remove.
    //     let idx = match self.rooms_metadata.iter().position(|m| m.id == room_id) {
    //         Some(i) => i,
    //         None => return, // nothing to delete
    //     };

    //     // Remove the metadata entry.
    //     self.rooms_metadata.remove(idx);

    //     // Re‑compute adjacency for the remaining rooms.
    //     let len = self.rooms_metadata.len();

    //     for i in 0..len {
    //         // Split the vector into three parts
    //         let (before, rest) = self.rooms_metadata.split_at_mut(i);
    //         let (room_i, after) = rest.split_first_mut().unwrap(); // safe because i < len

    //         // Clear the old adjacency list.
    //         room_i.adjacent_rooms.clear();

    //         // Compare with rooms that come i
    //         for other in before.iter() {
    //             if Self::are_rooms_adjacent(room_i, other) {
    //                 room_i.adjacent_rooms.push(other.id);
    //             }
    //         }

    //         // Compare with rooms that come after i
    //         for other in after.iter() {
    //             if Self::are_rooms_adjacent(room_i, other) {
    //                 room_i.adjacent_rooms.push(other.id);
    //             }
    //         }
    //     }
    // }

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
}