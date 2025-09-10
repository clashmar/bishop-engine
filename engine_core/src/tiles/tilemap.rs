// engine_core/src/tiles/tilemap.rs
use serde_with::{serde_as, FromInto};
use crate::assets::asset_manager::{AssetManager};
use crate::constants::*;
use crate::ecs::entity::Entity;
use crate::ecs::world_ecs::WorldEcs;
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
    pub tiles: Vec<Vec<Tile>>,
    #[serde_as(as = "FromInto<[f32; 4]>")]
    pub background: Color,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![Tile::default(); width]; height],
            background: LIGHTGRAY,
        }
    }

    pub fn draw(
        &self,
        camera: &Camera2D,
        exits: &Vec<Exit>,
        world_ecs: &WorldEcs,
        asset_manager: &mut AssetManager,
    ) {
        clear_background(BLACK);
        set_camera(camera);

        // background rectangle (unchanged)
        draw_rectangle(
            0.0,
            0.0,
            self.width as f32 * TILE_SIZE,
            self.height as f32 * TILE_SIZE,
            self.background,
        );

        for y in 0..self.height {
            for x in 0..self.width {
                let tile_inst = &self.tiles[y][x];
                if tile_inst.entity == Entity::null() {
                    continue;
                }

                // Sprite component (visual)
                if let Some(tile_sprite) = world_ecs.get::<TileSprite>(tile_inst.entity) {
                    let tex = asset_manager.get_texture_from_id(tile_sprite.sprite_id);
                    let dest = vec2(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                    draw_texture_ex(
                            tex,
                            dest.x,
                            dest.y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                                ..Default::default()
                            },
                        );
                }
            }
        }
        self.draw_exits(exits);
    }

    fn draw_exits(&self, exits: &Vec<Exit>) {
        for exit in exits {
            self.draw_exit(exit.position, exit.direction);
        }
    }

    /// Draw a yellow exit overlay/arrow at the given position
    pub fn draw_exit(&self, pos: Vec2, direction: ExitDirection) {
        let tile_size = TILE_SIZE;

        // Position in world coordinates, including outside tiles
        let x = pos.x * tile_size;
        let y = pos.y * tile_size;

        // Draw semi-transparent rectangle
        draw_rectangle(x, y, tile_size, tile_size, LIGHTGRAY);

        let arrow_center = vec2(x + tile_size / 2.0, y + tile_size / 2.0);
        let arrow_color = Color::new(1.0, 1.0, 0.0, 1.0);

        let offsets = match direction {
            ExitDirection::Up => [vec2(0.0, -1.0), vec2(-1.0, 1.0), vec2(1.0, 1.0)],
            ExitDirection::Down => [vec2(0.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, -1.0)],
            ExitDirection::Left => [vec2(-1.0, 0.0), vec2(1.0, -1.0), vec2(1.0, 1.0)],
            ExitDirection::Right => [vec2(1.0, 0.0), vec2(-1.0, -1.0), vec2(-1.0, 1.0)],
        };

        draw_triangle(
            arrow_center + offsets[0] * tile_size / 4.0,
            arrow_center + offsets[1] * tile_size / 4.0,
            arrow_center + offsets[2] * tile_size / 4.0,
            arrow_color
        );
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

pub fn tile_to_world(grid_position: GridPos, map_height: usize) -> Vec2 {
    Vec2::new(
        grid_position.x() as f32 * TILE_SIZE,
        (map_height as f32 - 1.0 - grid_position.y() as f32) * TILE_SIZE,
    )
}