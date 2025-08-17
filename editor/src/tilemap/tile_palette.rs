use core::tile::{Tile};
use macroquad::prelude::*;
use crate::gui::ui_element::TilemapUiElement;

pub struct TilePalette {
    pub position: Vec2, // Top left corner of the palette
    pub tile_size: f32,
    pub columns: usize,        
    pub rows: usize,           
    pub selected_index: usize, 
    pub tiles: Vec<Tile>,
}

impl TilePalette {
    pub fn new(position: Vec2, tile_size: f32, columns: usize, rows: usize) -> Self {
        let tiles = vec![
            Tile::floor(),
            Tile::platform(),
            Tile::decoration(),
        ];

        Self {
            position,
            tile_size,
            columns,
            rows,
            selected_index: 0,
            tiles,
        }
    }
}

impl TilemapUiElement for TilePalette {
    fn draw(&self, _camera: &Camera2D) {
        // Loop through each tile in the palette
        for (i, tile) in self.tiles.iter().enumerate() {
            let col = i % self.columns;
            let row = i / self.columns;

            let x = self.position.x + col as f32 * self.tile_size;
            let y = self.position.y + row as f32 * self.tile_size;

            // Draw tile's color
            draw_rectangle(x, y, self.tile_size, self.tile_size, tile.color);

            // Draw selection highlight
            if i == self.selected_index {
                draw_rectangle_lines(
                    x, y,
                    self.tile_size, self.tile_size,
                    3.0, RED,
                );
            }
        }
    }

    fn is_mouse_over(&self, mouse_pos: Vec2, _camera: &Camera2D) -> bool {
        let width = self.columns as f32 * self.tile_size;
        let height = self.rows as f32 * self.tile_size;
        Rect::new(self.position.x, self.position.y, width, height)
            .contains(mouse_pos)
    }

    fn on_click(
        &mut self,
        selected_tile: &mut Tile, 
        mouse_pos: Vec2, 
        camera: &Camera2D,
    ) {
        if self.is_mouse_over(mouse_pos, camera) {
            let local_x = mouse_pos.x - self.position.x;
            let local_y = mouse_pos.y - self.position.y;

            let col = (local_x / self.tile_size) as usize;
            let row = (local_y / self.tile_size) as usize;

            let index = row * self.columns + col;
            if index < self.tiles.len() {
                self.selected_index = index;
                *selected_tile = self.tiles[index];
            }
        }
    }
}