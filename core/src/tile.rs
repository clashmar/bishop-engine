use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Air,
    Floor,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub color: Color,
    pub tile_type: TileType,
}

impl Tile {
    pub fn new(color: Color, tile_type: TileType) -> Self {
        Tile { color, tile_type }
    }

    pub fn air() -> Self {
        Tile {
            color: GRAY,
            tile_type: TileType::Air,
        }
    }

    pub fn floor() -> Self {
        Tile {
            color: DARKGRAY,
            tile_type: TileType::Floor,
        }
    }
}