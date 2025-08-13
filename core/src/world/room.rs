use macroquad::prelude::*;
use crate::{tile::GridPos, tilemap::TileMap};

pub struct Room {
    pub name: String,
    pub position: Vec2,      
    pub variants: Vec<RoomVariant>,  
    pub exits: Vec<Exit>,  
}

impl Default for Room {
    fn default() -> Self {
        Self {
            name: String::new(),
            position: Vec2::ZERO,
            variants: vec![RoomVariant::default()],
            exits: vec![],
        }
    }
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

pub struct Exit {
    pub position: GridPos,     
    pub target_map: String,    
    pub target_position: GridPos,
}