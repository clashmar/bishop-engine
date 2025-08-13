use crate::gui::resize_button::ResizeButton;
use crate::gui::{ui_element::UiElement};
use crate::tilemap::tile_palette::{TilePalette};
use macroquad::prelude::*;
use rfd::FileDialog;
use core::tilemap::TileMap;
use core::tile::{Tile, GridPos, TileType};
use core::constants::*;
use std::fs::File;
use std::path::Path;
use std::io::Write;

pub struct TileMapEditor {
    scaling_ui_elements: Vec<Box<dyn UiElement>>,
    static_ui_elements: Vec<Box<dyn UiElement>>,
    selected_tile: Tile,
    camera: Camera2D,
    show_grid: bool,
    ui_clicked: bool,
    initialized: bool, 
}

impl TileMapEditor  {
    pub fn new() -> Self {
        let camera = Camera2D::default();

        let mut static_ui_elements: Vec<Box<dyn UiElement>> = Vec::new();

        static_ui_elements.push(Box::new(TilePalette::new(
            vec2(10.0, 10.0),
            32.0,
            2,
            2,
        )));

        let editor = Self {
            scaling_ui_elements: Vec::new(),
            static_ui_elements,
            selected_tile: Tile::floor(),
            camera,
            show_grid: true,
            ui_clicked: false,
            initialized: false,
        };

        editor
    }

    /// Update the editor with a mutable reference to the map
    pub fn update(&mut self, map: &mut TileMap) {
        if !self.initialized {
            self.reset_camera_view(map);
            self.initialized = true;
        }

        self.scaling_ui_elements.clear();
        ResizeButton::build_all(map, &mut self.scaling_ui_elements);

        let mouse_pos = mouse_position().into();
        self.handle_camera_controls();
        self.handle_ui_clicks(mouse_pos, map);

        if !self.ui_clicked {
            self.handle_tile_placement(mouse_pos, map);
        }

        if is_key_pressed(KeyCode::R) {
            self.reset_camera_view(map);
        }
        if is_key_pressed(KeyCode::G) {
            self.show_grid = !self.show_grid;
        }

        self.handle_save_map(map);
    }

    pub fn draw(&self, map: &TileMap) {
        clear_background(WHITE);
        set_camera(&self.camera);

        self.draw_map(map);
        self.draw_grid(map);
        self.draw_hover_highlight(map);

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

    fn handle_ui_clicks(&mut self, mouse_pos: Vec2, map: &mut TileMap) {
        if is_mouse_button_pressed(MouseButton::Left) {
            for element in &mut self.scaling_ui_elements {
                if element.is_mouse_over(mouse_pos, &self.camera) {
                    element.on_click(map, &mut self.selected_tile, mouse_pos, &self.camera);
                    self.ui_clicked = true;
                    break;
                }
            }

            for element in &mut self.static_ui_elements {
                if element.is_mouse_over(mouse_pos, &self.camera) {
                    element.on_click(map, &mut self.selected_tile, mouse_pos, &self.camera);
                    self.ui_clicked = true;
                    break;
                }
            }
        }

        // Unblock UI
        if is_mouse_button_released(MouseButton::Left) {
            self.ui_clicked = false;
        }
    }

    fn handle_tile_placement(&mut self, mouse_pos: Vec2, map: &mut TileMap) {
        let mouse_over_ui = self.is_mouse_over_ui(mouse_pos);
        let hover_pos = self.get_hovered_tile(map);

        if !mouse_over_ui {
            if is_mouse_button_down(MouseButton::Left) {
                if let Some(pos) = hover_pos {
                    if let Some((x, y)) = pos.as_usize() {
                        map.tiles[y][x] = self.selected_tile.clone();
                    }
                }
            }

            if is_mouse_button_down(MouseButton::Right) {
                if let Some(pos) = hover_pos {
                    if let Some((x, y)) = pos.as_usize() {
                        map.tiles[y][x] = Tile::none();
                    }
                }
            }
        }
    }

    fn handle_save_map(&self, map: &TileMap) {
        if is_key_pressed(KeyCode::S) {
            if let Some(path) = FileDialog::new()
                .add_filter("Map files", &["map"])
                .set_file_name("untitled.map")
                .save_file()
            {
                if let Err(e) = self.save_to_file(map, &path) {
                    eprintln!("Failed to save map: {}", e);
                } else {
                    println!("Map saved to {:?}", path);
                }
            }
        }
    }

    fn draw_map(&self, map: &TileMap) {
        draw_rectangle(
            0.0,
            0.0,
            map.width as f32 * TILE_SIZE,
            map.height as f32 * TILE_SIZE,
            map.background,
        );

        for y in 0..map.height {
            for x in 0..map.width {
                let tile = &map.tiles[y][x];
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

    fn draw_grid(&self, map: &TileMap) {
        if !self.show_grid {
            return;
        }

        let zoom_scale = self.camera.zoom.x.abs();
        let base_width = 0.5;
        let min_line_width = 2.0;
        let max_line_width = 5.0;
        let line_width = (base_width / zoom_scale).clamp(min_line_width, max_line_width);
        let grid_color = Color::from_rgba(0, 0, 0, 20);

        for y in 0..=map.height {
            draw_line(
                0.0,
                y as f32 * TILE_SIZE,
                map.width as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                line_width,
                grid_color,
            );
        }

        for x in 0..=map.width {
            draw_line(
                x as f32 * TILE_SIZE,
                0.0,
                x as f32 * TILE_SIZE,
                map.height as f32 * TILE_SIZE,
                line_width,
                grid_color,
            );
        }
    }

    fn draw_hover_highlight(&self, map: &TileMap) {
        if let Some(tile_pos) = self.get_hovered_tile(map) {
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
        // Draw scaling UI
        for element in &self.scaling_ui_elements {
            element.draw(&self.camera);
        }
        
        // Reset to default camera for static UI drawing
        set_default_camera();

        // Draw static UI
        for element in &self.static_ui_elements {
            element.draw(&self.camera);
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, map: &TileMap, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        for row in map.tiles.iter().rev() {
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

    fn get_hovered_tile(&self, map: &TileMap) -> Option<GridPos> {
        let mouse_pos: Vec2 = mouse_position().into();
        let world_pos = self.camera.screen_to_world(mouse_pos);
        let pos = GridPos::from_world(world_pos);

        if pos.is_in_bounds(map.width, map.height) {
            Some(pos)
        } else {
            None
        }
    }

    pub fn reset_camera_view(&mut self, map: &TileMap) {
        let aspect_x = 2.0 / screen_width();
        let aspect_y = -2.0 / screen_height();
        let initial_scale = 1.0 / 2.0;

        self.camera.target = vec2(
            (map.width as f32 * TILE_SIZE) / 2.0,
            (map.height as f32 * TILE_SIZE) / 2.0,
        );
        self.camera.zoom = vec2(aspect_x * initial_scale, aspect_y * initial_scale);
    }

    fn is_mouse_over_ui(&self, mouse_pos: Vec2) -> bool {
        self.scaling_ui_elements
        .iter()
        .any(|element| element.is_mouse_over(mouse_pos, &self.camera))
    }

    pub fn reset(&mut self) {
        self.initialized = false
    }
}