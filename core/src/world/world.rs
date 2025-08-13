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
            exits: Vec::new(),
        };
        self.rooms.push(room);
        self.rooms.len() - 1
    }
}