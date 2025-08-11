use macroquad::prelude::*;
use rfd::FileDialog;
use core::map::TileMap;
use core::tile::{Tile, GridPos, TileType};
use core::constants::*;
use std::fs::File;
use std::path::Path;
use std::io::Write;

const TILE_OPTION_ICON_X: f32 = 10.0;
const TILE_OPTION_ICON_Y: f32 = 170.0;
const TILE_OPTION_ICON_WIDTH: f32 = 100.0;
const TILE_OPTION_ICON_HEIGHT: f32 = 25.0;
const TILE_OPTION_ICON_PADDING: f32 = 10.0;

pub struct TileMapEditor {
    ui_elements: Vec<Rect>,
    tile_options: Vec<TileOption>,
    map: TileMap,
    selected_tile: Tile,
    camera: Camera2D,
    show_grid: bool,
}

struct TileOption {
    name: &'static str,
    tile: Tile,
    rect: Rect, 
}

impl TileMapEditor {
    pub fn new(width: usize, height: usize) -> Self {
        let camera = Camera2D::default();

        // Define tile options without rects yet (rects will be assigned dynamically)
        let tile_options = vec![
            TileOption { name: "Floor", tile: Tile::floor(), rect: Rect::default() },
            TileOption { name: "Platform", tile: Tile::platform(), rect: Rect::default() },
            TileOption { name: "Decoration", tile: Tile::decoration(), rect: Rect::default() },
        ];

        let mut state = Self {
            tile_options,
            ui_elements: Vec::new(),
            map: TileMap::new(width, height),
            selected_tile: Tile::floor(),
            camera,
            show_grid: true,
        };

        // Initialize camera view
        state.reset_camera_view();
        state
    }

    pub fn update(&mut self) {
        // Clear UI elements from last frame
        self.ui_elements.clear();

        for (i, option) in self.tile_options.iter_mut().enumerate() {
            let y = TILE_OPTION_ICON_Y + (TILE_OPTION_ICON_HEIGHT + TILE_OPTION_ICON_PADDING) * i as f32;
            option.rect = Rect::new(TILE_OPTION_ICON_X, y, TILE_OPTION_ICON_WIDTH, TILE_OPTION_ICON_HEIGHT);
            self.ui_elements.push(option.rect);
        }
        
        self.handle_camera_controls();

        let mouse_pos = mouse_position().into();
        self.handle_tile_selection(mouse_pos);
        self.handle_tile_placement(mouse_pos);
        self.handle_map_resizing();

        // Reset camera view with R key
        if is_key_pressed(KeyCode::R) {
            self.reset_camera_view();
        }
        
        // Toggle grid
        if is_key_pressed(KeyCode::G) {
            self.show_grid = !self.show_grid;
        }

        self.handle_save_map();
        
    }

    pub fn draw(&self) {
        clear_background(WHITE);

        // Set the camera before drawing
        set_camera(&self.camera);

        self.draw_map();
        self.draw_grid();
        self.draw_hover_highlight();

        // Reset to default camera for UI drawing
        set_default_camera();
        self.draw_ui();
    }

    fn handle_camera_controls(&mut self) {
        // Handle zoom
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

        // Handle pan
        if is_mouse_button_down(MouseButton::Middle) || is_mouse_button_down(MouseButton::Right) {
            let delta = mouse_delta_position();
            self.camera.target -= delta / self.camera.zoom;
        }
    }

    fn handle_tile_selection(&mut self, mouse_pos: Vec2) {
        if is_mouse_button_pressed(MouseButton::Left) {
            // Check which tile option button was clicked
            for option in &self.tile_options {
                if option.rect.contains(mouse_pos) {
                    self.selected_tile = option.tile.clone();
                    break;
                }
            }
        }
    }

    fn handle_tile_placement(&mut self, mouse_pos: Vec2) {
        // Place tiles
        if is_mouse_button_down(MouseButton::Left) && !self.is_mouse_over_ui(mouse_pos) {
            if let Some(tile_pos) = self.get_hovered_tile() {
                if let Some((x, y)) = tile_pos.as_usize() {
                    self.map.tiles[y][x] = self.selected_tile.clone();
                }
            }
        }

        // Remove tiles
        if is_mouse_button_down(MouseButton::Right) {
            if let Some(tile_pos) = self.get_hovered_tile() {
                if let Some((x, y)) = tile_pos.as_usize() {
                    self.map.tiles[y][x] = Tile::none();
                }
            }
        }
    }

    fn handle_map_resizing(&mut self) {
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
    }

    fn handle_save_map(&mut self) {
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

    fn draw_map(& self) {
        // Background
        draw_rectangle(
            0.0,
            0.0,
            self.map.width as f32 * TILE_SIZE,
            self.map.height as f32 * TILE_SIZE,
            self.map.background,
        );

        // Tiles
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
    }

    fn draw_grid(&self) {
        if !self.show_grid {
            return;
        }

        let zoom_scale = self.camera.zoom.x.abs();
        let base_width = 0.5; 
        let min_line_width = 2.0;
        let max_line_width = 5.0;
        let line_width = (base_width / zoom_scale).clamp(min_line_width, max_line_width);

        let grid_color = Color::from_rgba(0, 0, 0, 20);

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

    fn draw_hover_highlight(&self) {
        if let Some(tile_pos) = self.get_hovered_tile() {
            let zoom_scale = self.camera.zoom.x.abs();
            let base_width = 0.5; 
            let min_line_width = 2.0;
            let max_line_width = 5.0;
            let line_width = (base_width / zoom_scale).clamp(min_line_width, max_line_width);

            draw_rectangle_lines(
                tile_pos.x() as f32 * TILE_SIZE,
                tile_pos.y() as f32 * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                line_width,
                RED,
            );
        }
    }

    fn draw_ui(&self) {
        for option in &self.tile_options {
            let is_selected = option.tile.tile_type == self.selected_tile.tile_type;
            let color = if is_selected { GREEN } else { LIGHTGRAY };
            draw_rectangle(option.rect.x, option.rect.y, option.rect.w, option.rect.h, color);
            draw_text(option.name, option.rect.x + 10.0, option.rect.y + 17.0, 20.0, BLACK);
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        for row in self.map.tiles.iter().rev() {
            for tile in row {
                let c = match tile.tile_type {
                    TileType::None => '.',
                    TileType::Floor => '#',
                    TileType::Platform => '-',
                    TileType::Decoration => '*',
                };
                write!(file, "{}", c)?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    fn get_hovered_tile(&self) -> Option<GridPos> {
        let mouse_pos = mouse_position().into();
        let world_pos = self.camera.screen_to_world(mouse_pos);

        let pos = GridPos::from_world(world_pos);
        if pos.in_bounds(self.map.width, self.map.height) {
            Some(pos)
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

    // Returns true if the mouse is currently over any UI element that should block tile placement
    fn is_mouse_over_ui(&self, mouse_pos: Vec2) -> bool {
        self.ui_elements.iter().any(|rect| rect.contains(mouse_pos))
    }
}