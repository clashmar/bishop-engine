// engine_core/src/world/world.rs
use crate::tiles::tilemap::TileMap;
use crate::constants::TILE_SIZE;
use crate::ecs::{world_ecs::WorldEcs};
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
    pub world_ecs: WorldEcs,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos(pub IVec2);

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        GridPos(IVec2::new(x, y))
    }

    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }

    /// Check if this position is within map bounds
    pub fn is_in_bounds(&self, width: usize, height: usize) -> bool {
        self.0.x >= 0
            && self.0.y >= 0
            && self.0.x < width as i32
            && self.0.y < height as i32
    }

    /// Convert from world coordinates to tile coordinates
    pub fn from_world(world_pos: Vec2) -> Self {
        GridPos::new(
            (world_pos.x / TILE_SIZE) as i32,
            (world_pos.y / TILE_SIZE) as i32,
        )
    }

    /// Convert to usize tuple (if valid)
    pub fn as_usize(&self) -> Option<(usize, usize)> {
        if self.0.x >= 0 && self.0.y >= 0 {
            Some((self.0.x as usize, self.0.y as usize))
        } else {
            None
        }
    }
    
    pub fn from_world_edge(world_pos: Vec2, map: &TileMap) -> Self {
        let mut x = (world_pos.x / TILE_SIZE).floor() as i32;
        let mut y = (world_pos.y / TILE_SIZE).floor() as i32;

        // Snap to map edges
        if x < 0 { x = -1; }
        else if x >= map.width as i32 { x = map.width as i32; }

        if y < 0 { y = -1; }
        else if y >= map.height as i32 { y = map.height as i32; }

        GridPos::new(x, y)
    }
}