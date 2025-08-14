use macroquad::prelude::*;
use crate::{constants::TILE_SIZE, tilemap::TileMap};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    None,
    Floor,
    Platform,
    Decoration,
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
            color: BLANK,
            is_walkable: false,
            is_solid: false,
        }
    }

    pub fn platform() -> Self {
        Tile {
            tile_type: TileType::Platform,
            color: DARKGRAY,
            is_walkable: true,
            is_solid: false,
        }
    }
    
    pub fn floor() -> Self {
        Tile {
            tile_type: TileType::Floor,
            color: BLACK,
            is_walkable: true,
            is_solid: true,
        }
    }

    pub fn decoration() -> Self {
        Tile {
            tile_type: TileType::Decoration,
            color: YELLOW,
            is_walkable: false,
            is_solid: false,
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