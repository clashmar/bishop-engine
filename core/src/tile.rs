use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    None,
    Floor,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub tile_type: TileType,
    pub color: Color,
    pub is_walkable: bool,
    pub is_solid: bool,
}

impl Tile {
    pub fn none() -> Self {
        Tile {
            tile_type: TileType::None,
            color: GRAY,
            is_walkable: false,
            is_solid: false,
        }
    }
    
    pub fn floor() -> Self {
        Tile {
            tile_type: TileType::Floor,
            color: DARKGRAY,
            is_walkable: true,
            is_solid: true,
        }
    }
}