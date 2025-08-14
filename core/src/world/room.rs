use macroquad::prelude::*;
use crate::{tilemap::TileMap, world::world::World};

pub struct Room {
    pub name: String,
    pub position: Vec2,      
    pub variants: Vec<RoomVariant>,  
    pub exits: Vec<Exit>,
    pub adjacent_rooms: Vec<usize>,   
}

impl Room {
    pub fn size(&self) -> Vec2 {
        if let Some(first_variant) = self.variants.first() {
            Vec2::new(
                first_variant.tilemap.width as f32,
                first_variant.tilemap.height as f32,
            )
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    pub fn link_exits_slice(&mut self, other_rooms: &[&Room]) {
        //
    }

    pub fn world_exit_positions(&self) -> Vec<(Vec2, ExitDirection)> {
        self.exits.iter().map(|exit| {
            (self.position + exit.position, exit.direction)
        }).collect()
    }
}

pub struct RoomVariant {
    pub id: String,
    pub tilemap: TileMap,      
}

impl Default for RoomVariant {
    fn default() -> Self {
        Self {
            id: String::new(),
            tilemap: TileMap::new(10, 10),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ExitDirection {
    Up,
    Right,
    Down,
    Left
}

#[derive(Clone)]
pub struct Exit {
    pub position: Vec2,                 
    pub direction: ExitDirection,      
    pub target_room_id: Option<usize>, 
}