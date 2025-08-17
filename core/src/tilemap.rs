use serde_with::{serde_as, FromInto};
use std::io::BufRead;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::constants::*;
use crate::tile::{GridPos, Tile, TileType};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    #[serde_as(as = "FromInto<[f32; 4]>")]
    pub background: Color,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![Tile::none(); width]; height],
            background: LIGHTGRAY,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut tiles: Vec<Vec<Tile>> = Vec::new();
        for line in reader.lines() {
        let row: Vec<Tile> = line?
            .chars()
            .map(|c| match c {
                '#' => Tile::floor(),
                '-' => Tile::platform(),
                '*' => Tile::decoration(),
                '.' => Tile::none(),
                _   => Tile::none(),
            })
            .collect();
        tiles.push(row);
    }

        let height = tiles.len();
        let width = tiles.get(0).map_or(0, |r| r.len());

        Ok(Self {
            width,
            height,
            tiles: tiles.into_iter().rev().collect(),
            background: LIGHTGRAY,
        })
    }

    pub fn draw(&self) {
        // Draw the background
        draw_rectangle(
            0.0,
            0.0,
            self.width as f32 * TILE_SIZE,
            self.height as f32 * TILE_SIZE,
            self.background,
        );

        // Draw tiles on top, skipping None tiles
        for (y, row) in self.tiles.iter().rev().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.tile_type != TileType::None {
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
    }

    pub fn get_tile(&self, pos: GridPos) -> Option<&Tile> {
        let (x, y) = pos.as_usize()?;
        self.tiles.get(y)?.get(x)
    }

    pub fn pixel_to_grid(pixel: f32) -> i32 {
        (pixel / TILE_SIZE).floor() as i32
    }

    pub fn any_tiles_in_range<F>(
        map: &TileMap,
        x_range: std::ops::RangeInclusive<i32>,
        y_range: std::ops::RangeInclusive<i32>,
        predicate: F,
    ) -> bool
    where
        F: Fn(&Tile) -> bool,
    {
        let y_start = *y_range.start();
        let y_end = *y_range.end();

        for x in x_range {
            for y in y_start..=y_end {
                let pos = GridPos::new(x, y);
                if pos.is_in_bounds(map.width, map.height) {
                    if let Some(tile) = map.get_tile(pos) {
                        if predicate(tile) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

pub fn get_current_map() -> TileMap {
    let map_dir = PathBuf::from("game/src/maps");

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

    TileMap::new(10, 10)
}

pub fn tile_to_world(grid_position: GridPos, map_height: usize) -> Vec2 {
    Vec2::new(
        grid_position.x() as f32 * TILE_SIZE,
        (map_height as f32 - 1.0 - grid_position.y() as f32) * TILE_SIZE,
    )
}