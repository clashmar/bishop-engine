use std::io::BufRead;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::constants::*;
use crate::tile::{Tile, TileType};
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl TileMap {
    pub fn new(tiles: Vec<Vec<Tile>>) -> Self {
        let height = tiles.len();
        let width = tiles.get(0).map_or(0, |row| row.len());

        Self { width, height, tiles }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut tiles: Vec<Vec<Tile>> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let row: Vec<Tile> = line
                .chars()
                .map(|c| {
                    let tile_type = match c {
                        '.' => TileType::Air,
                        '#' => TileType::Floor,
                        _ => TileType::Air, // default to air if unknown
                    };
                    let color = match tile_type {
                        TileType::Air => GRAY,
                        TileType::Floor => DARKGRAY,
                    };
                    Tile::new(color, tile_type)
                })
                .collect();
            tiles.push(row);
        }

        let height = tiles.len();
        let width = tiles.first().map_or(0, |row| row.len());

        Ok(TileMap {
            width,
            height,
            tiles: tiles.into_iter().rev().collect(), // flip vertically to match drawing
        })
    }

    pub fn draw(&self) {
        for (y, row) in self.tiles.iter().rev().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                draw_rectangle(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    tile.color,
                );
            }
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }
}

pub fn get_current_map() -> TileMap {

    let map_dir = PathBuf::from("game/src/maps");

    // Log where we're looking
    match map_dir.canonicalize() {
        Ok(abs_path) => println!("Looking for map files in: {}", abs_path.display()),
        Err(e) => println!("Could not canonicalize path {:?}: {}", map_dir, e),
    }

    let maybe_file = fs::read_dir(&map_dir)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "txt" || path.extension()? == "map" {
                    Some(path)
                } else {
                    None
                }
            })
        });

    if let Some(path) = maybe_file {
        match TileMap::load_from_file(&path) {
            Ok(map) => return map,
            Err(e) => eprintln!("Failed to load map from {:?}: {}", path, e),
        }
    } else {
        eprintln!("No map files found in {:?}", map_dir);
    }

    // fallback to empty map
    TileMap::new(vec![vec![Tile::air(); 10]; 10])
}

pub fn tile_to_world(grid_position: IVec2, map_height: usize) -> Vec2 {
    Vec2::new(
        grid_position.x as f32 * TILE_SIZE,
        (map_height as f32 - 1.0 - grid_position.y as f32) * TILE_SIZE,
    )
}