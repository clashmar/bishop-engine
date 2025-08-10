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
    selected_walkable: bool,
    camera: Camera2D,
    show_grid: bool,
}

impl EditorState {
    pub fn new(width: usize, height: usize) -> Self {
        let camera = Camera2D::default();

        let mut state = Self {
            map: TileMap::new(width, height),
            selected_walkable: true,
            camera,
            show_grid: true,
        };

        // Initialize camera view
        state.reset_camera_view();
        state
    }

    pub fn update(&mut self) {
        // Handle zoom with mouse wheel
        let wheel = mouse_wheel().1;

        if wheel != 0.0 {
            let zoom_speed = 1.1;
            let zoom_factor = if wheel > 0.0 { zoom_speed } else { 1.0 / zoom_speed };

            // Change world scale by modifying zoom based on screen size
            let aspect_x = 2.0 / screen_width();
            let aspect_y = -2.0 / screen_height(); // negative to flip Y

            let current_scale = self.camera.zoom.x / aspect_x;

            let new_scale = (current_scale * zoom_factor)
                .clamp(0.25, 4.0); // Min and max zoom levels

            self.camera.zoom = vec2(aspect_x * new_scale, aspect_y * new_scale);
        }

        // Pan with middle/right mouse
        if is_mouse_button_down(MouseButton::Middle) || is_mouse_button_down(MouseButton::Right) {
            let delta = mouse_delta_position();
            self.camera.target -= delta / self.camera.zoom;
        }

        // Place tile (left click)
        if is_mouse_button_down(MouseButton::Left) {
            if let Some((tx, ty)) = self.get_hovered_tile() {
                if tx < self.map.width && ty < self.map.height {
                    self.map.tiles[ty][tx] = Tile::floor();
                }
            }
        }

        // Remove tile (right click)
        if is_mouse_button_down(MouseButton::Right) {
            if let Some((tx, ty)) = self.get_hovered_tile() {
                if tx < self.map.width && ty < self.map.height {
                    self.map.tiles[ty][tx] = Tile::none();
                }
            }
        }

        // Resize map
        if is_key_pressed(KeyCode::Up) {
            self.map.tiles.push(vec![Tile::none(); self.map.width]);
            self.map.height += 1;
        }
        if is_key_pressed(KeyCode::Down) && self.map.height > 1 {
            self.map.tiles.pop();
            self.map.height -= 1;
        }
        if is_key_pressed(KeyCode::Right) {
            for row in &mut self.map.tiles {
                row.push(Tile::none());
            }
            self.map.width += 1;
        }
        if is_key_pressed(KeyCode::Left) && self.map.width > 1 {
            for row in &mut self.map.tiles {
                row.pop();
            }
            self.map.width -= 1;
        }

        // Reset camera view with R key
        if is_key_pressed(KeyCode::R) {
            self.reset_camera_view();
        }
        
        // Toggle grid
        if is_key_pressed(KeyCode::G) {
            self.show_grid = !self.show_grid;
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
                    TileType::Floor => '#',
                    TileType::None => '.',
                };
                write!(file, "{}", c)?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    fn get_hovered_tile(&self) -> Option<(usize, usize)> {
        let mouse_pos = mouse_position();
        
        // Convert screen coordinates to world coordinates using the camera
        let world_pos = self.camera.screen_to_world(mouse_pos.into());
        
        let x = (world_pos.x / TILE_SIZE) as i32;
        let y = (world_pos.y / TILE_SIZE) as i32;
        
        // Check bounds
        if x >= 0 && y >= 0 && x < self.map.width as i32 && y < self.map.height as i32 {
            Some((x as usize, y as usize))
        } else {
            None
        }
    }

    // Helper to reset camera target and zoom based on current map size and screen size
    pub fn reset_camera_view(&mut self) {
        let aspect_x = 2.0 / screen_width();
        let aspect_y = -2.0 / screen_height();
        let initial_scale = 1.0 / 2.0; // adjust initial zoom scale as you like

        self.camera.target = vec2(
            (self.map.width as f32 * TILE_SIZE) / 2.0,
            (self.map.height as f32 * TILE_SIZE) / 2.0,
        );

        self.camera.zoom = vec2(aspect_x * initial_scale, aspect_y * initial_scale);
    }

    pub fn draw(&self) {
        clear_background(WHITE);

        // Set the camera before drawing
        set_camera(&self.camera);

        // Draw the TileMap background first
        draw_rectangle(
            0.0,
            0.0,
            self.map.width as f32 * TILE_SIZE,
            self.map.height as f32 * TILE_SIZE,
            self.map.background,
        );

        // Draw tiles
        // Draw tiles, skip None tiles
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let tile = &self.map.tiles[y][x];
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

        // Draw grid with zoom-aware line width
        let zoom_scale = self.camera.zoom.x.abs();
        let base_width = 0.5; // much thinner base
        let min_line_width = 2.0;
        let max_line_width = 5.0;

        // Clamp line width based on zoom, but never below min_line_width
        let line_width = (base_width / zoom_scale).clamp(min_line_width, max_line_width);

        if self.show_grid {
            // Semi-transparent black
            let grid_color = Color::from_rgba(0, 0, 0, 20);

            // Draw horizontal lines
            for y in 0..=self.map.height {
                draw_line(
                    0.0,
                    y as f32 * TILE_SIZE,
                    self.map.width as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    line_width,
                    grid_color,
                );
            }

            // Draw vertical lines
            for x in 0..=self.map.width {
                draw_line(
                    x as f32 * TILE_SIZE,
                    0.0,
                    x as f32 * TILE_SIZE,
                    self.map.height as f32 * TILE_SIZE,
                    line_width,
                    grid_color,
                );
            }
        }

        let highlight_color = Color::from_rgba(255, 0, 0, 255);

        // Highlight hovered tile
        if let Some((x, y)) = self.get_hovered_tile() {
            draw_rectangle_lines(
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                line_width,
                highlight_color,
            );
        }

        // Reset to default camera for UI drawing
        set_default_camera();
        
        // Draw UI elements (these will be drawn in screen space)
        draw_text(
            "Controls:",
            10.0,
            20.0,
            16.0,
            BLACK,
        );
        draw_text(
            "Mouse Wheel: Zoom",
            10.0,
            40.0,
            16.0,
            BLACK,
        );
        draw_text(
            "Right/Middle Mouse: Pan",
            10.0,
            60.0,
            16.0,
            BLACK,
        );
        draw_text(
            "R: Reset View",
            10.0,
            80.0,
            16.0,
            BLACK,
        );
        draw_text(
            "S: Save Map",
            10.0,
            100.0,
            16.0,
            BLACK,
        );
        draw_text(
            "Arrow Keys: Resize Map",
            10.0,
            120.0,
            16.0,
            BLACK,
        );
        draw_text(
            "G: Toggle Grid Lines",
            10.0,
            140.0,
            16.0,
            BLACK,
        );
    }
}