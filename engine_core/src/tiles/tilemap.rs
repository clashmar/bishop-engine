// engine_core/src/tiles/tilemap.rs
use std::collections::HashMap;
use serde_with::{serde_as, FromInto};
use crate::assets::asset_manager::{AssetManager};
use crate::ecs::component::Position;
use crate::ecs::world_ecs::WorldEcs;
use crate::global::tile_size;
use crate::tiles::tile::{Tile, TileSprite};
use crate::world::room::{Exit, ExitDirection};
use crate::world::world::GridPos;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tiles: HashMap<(usize, usize), Tile>,
    #[serde_as(as = "FromInto<[f32; 4]>")]
    pub background: Color,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: HashMap::new(),
            background: LIGHTGRAY,
        }
    }

    pub fn draw(
        &self,
        exits: &Vec<Exit>,
        world_ecs: &WorldEcs,
        asset_manager: &mut AssetManager,
        room_position: Vec2,
    ) {
        clear_background(BLACK);

        // Background
        draw_rectangle(
            room_position.x,
            room_position.y,
            self.width as f32 * tile_size(),
            self.height as f32 * tile_size(),
            self.background,
        );

        for ((x, y), tile) in &self.tiles {
            let tile_pos = vec2(*x as f32 * tile_size(), *y as f32 * tile_size()) + room_position;

            if let Some(sprite) = tile
                .entity                    
                .and_then(|entity| world_ecs.get::<TileSprite>(entity))
            {
                let tex = asset_manager.get_texture_from_id(sprite.sprite_id);
                draw_texture_ex(
                    tex,
                    tile_pos.x,
                    tile_pos.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(tile_size(), tile_size())),
                        ..Default::default()
                    },
                );
            }
        }
        self.draw_exits(exits, room_position);
    }

    fn draw_exits(&self, exits: &Vec<Exit>, room_position: Vec2) {
        for exit in exits {
            let position = exit.position * tile_size() + room_position;
            self.draw_exit(position, exit.direction);
        }
    }

    /// Draw a yellow exit overlay/arrow at the given position
    pub fn draw_exit(
        &self, 
        position: Vec2, 
        direction: ExitDirection,
    ) {
        // Position in world coordinates, including outer tiles
        let x = position.x;
        let y = position.y;

        // Draw semi-transparent rectangle
        draw_rectangle(x, y, tile_size(), tile_size(), LIGHTGRAY);

        let arrow_center = vec2(x + tile_size() / 2.0, y + tile_size() / 2.0);
        let arrow_color = Color::new(1.0, 1.0, 0.0, 1.0);

        let offsets = match direction {
            ExitDirection::Up => [vec2(0.0, -1.0), vec2(-1.0, 1.0), vec2(1.0, 1.0)],
            ExitDirection::Down => [vec2(0.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, -1.0)],
            ExitDirection::Left => [vec2(-1.0, 0.0), vec2(1.0, -1.0), vec2(1.0, 1.0)],
            ExitDirection::Right => [vec2(1.0, 0.0), vec2(-1.0, -1.0), vec2(-1.0, 1.0)],
        };

        draw_triangle(
            arrow_center + offsets[0] * tile_size() / 4.0,
            arrow_center + offsets[1] * tile_size() / 4.0,
            arrow_center + offsets[2] * tile_size() / 4.0,
            arrow_color
        );
    }

    /// Insert a tile at a grid coordinate.
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles.insert((x, y), tile);
    }


    /// Retrieve a tile, returning `None` for empty cells.
    pub fn get_tile(&self, pos: GridPos) -> Option<&Tile> {
        let (x, y) = pos.as_usize()?;
        self.tiles.get(&(x, y))
    }

    pub fn pixel_to_grid(pixel: f32) -> i32 {
        (pixel / tile_size()).floor() as i32
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

    /// Remove a tile from the map and ECS.
    pub fn remove_tile(
        &mut self,
        grid_position: (usize, usize),
        world_ecs: &mut WorldEcs,
    ) {
        if let Some(tile) = self.tiles.remove(&grid_position) {
            if let Some(entity) = tile.entity {
                world_ecs.remove_entity(entity);
            }
        }
    }
}

pub fn tile_to_world(grid_position: GridPos) -> Vec2 {
    Vec2::new(
        grid_position.x() as f32 * tile_size(),
        grid_position.y() as f32 * tile_size(),
    )
}

/// Shift every tile in a tilemap by (dx, dy) and updats ECS positions.
pub fn shift_tiles(
    map: &mut TileMap,
    dx: isize,
    dy: isize,
    world_ecs: &mut WorldEcs,
) {
    // Nothing to do if the offset is zero
    if dx == 0 && dy == 0 {
        return;
    }

    // Take the current tiles out of the map, then re‑insert them with the offset
    let old_tiles = std::mem::take(&mut map.tiles);

    for ((x, y), tile) in old_tiles {
        // New grid coordinates.
        let nx = (x as isize + dx) as usize;
        let ny = (y as isize + dy) as usize;

        // Update the position component
        if let Some(entity) = tile.entity {
            if let Some(pos) = world_ecs.get_mut::<Position>(entity) {
                pos.position.x += dx as f32 * tile_size();
                pos.position.y += dy as f32 * tile_size();
            }
        }

        // Re‑insert the tile at its new grid location
        map.tiles.insert((nx, ny), tile);
    }
}