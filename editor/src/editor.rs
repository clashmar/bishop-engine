use macroquad::prelude::*;
use rfd::FileDialog;
use core::map::TileMap;
use core::tile::{Tile, TileType};
use core::constants::*;
use std::fs::File;
use std::path::Path;
use std::io::Write;

pub struct EditorState {
    map: TileMap,
    selected_tile_type: TileType,
}

impl EditorState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            map: TileMap::new(vec![vec![Tile::air(); width]; height]),
            selected_tile_type: TileType::Floor,
        }
    }

    pub fn update(&mut self) {
        // Click to toggle tile
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some((x, y)) = self.get_hovered_tile() {
                if y < self.map.tiles.len() && x < self.map.tiles[0].len() {
                    let tile = &mut self.map.tiles[y][x];
                    tile.tile_type = match tile.tile_type {
                        TileType::Air => TileType::Floor,
                        TileType::Floor => TileType::Air,
                    };
                    tile.color = match tile.tile_type {
                        TileType::Air => GRAY,
                        TileType::Floor => DARKGRAY,
                    };
                }
            }
        }

        // Optional: resize map with keys
        if is_key_pressed(KeyCode::Up) {
            self.map.tiles.push(vec![Tile::air(); self.map.width]);
            self.map.height += 1;
        }
        if is_key_pressed(KeyCode::Down) && self.map.height > 1 {
            self.map.tiles.pop();
            self.map.height -= 1;
        }
        if is_key_pressed(KeyCode::Right) {
            for row in &mut self.map.tiles {
                row.push(Tile::air());
            }
            self.map.width += 1;
        }
        if is_key_pressed(KeyCode::Left) && self.map.width > 1 {
            for row in &mut self.map.tiles {
                row.pop();
            }
            self.map.width -= 1;
        }

        // Save map
        if is_key_pressed(KeyCode::S) {
            if let Some(path) = FileDialog::new()
            .add_filter("Map files", &["map"])
            .set_file_name("untitled.map")
            .save_file()
            {
                if let Err(e) = self.save_to_file(&path) {
                    eprintln!("Failed to save map: {}", e);
                } else {
                    println!("Map saved to {:?}", path);
                }
            }
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        for row in self.map.tiles.iter().rev() {
            for tile in row {
                let c = match tile.tile_type {
                    TileType::Air => '.',
                    TileType::Floor => '#',
                };
                write!(file, "{}", c)?;
            }
            writeln!(file)?;
        }

        Ok(())
    }

    fn get_hovered_tile(&self) -> Option<(usize, usize)> {
        let mouse_pos = mouse_position();
        let x = (mouse_pos.0 / TILE_SIZE) as usize;
        let y = (mouse_pos.1 / TILE_SIZE) as usize;
        let flipped_y = self.map.height.saturating_sub(1).saturating_sub(y);
        Some((x, flipped_y))
    }

    pub fn draw(&self) {
        clear_background(LIGHTGRAY);
        self.map.draw();

        // Draw grid
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                draw_rectangle_lines(
                    x as f32 * TILE_SIZE,
                    (self.map.height - 1 - y) as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    1.0,
                    BLACK,
                );
            }
        }

        // Highlight hovered tile
        if let Some((x, y)) = self.get_hovered_tile() {
            draw_rectangle_lines(
                x as f32 * TILE_SIZE,
                (self.map.height - 1 - y) as f32 * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                2.0,
                RED,
            );
        }
    }
}