use serde_with::FromInto;
use uuid::Uuid;
use crate::{world::room::{RoomMetadata}};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct World {
    pub id: Uuid,
    pub name: String,
    pub rooms_metadata: Vec<RoomMetadata>,
    pub starting_room: Option<Uuid>,
    #[serde_as(as = "Option<FromInto<[f32; 2]>>")]
    pub starting_position: Option<Vec2>,
}

impl World {
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