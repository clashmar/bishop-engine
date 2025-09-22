// editor/src/tilemap/tilemap_editor.rs
use macroquad::prelude::*;
use crate::{gui::{
    resize_button::ResizeButton,
    ui_element::{DynamicTilemapUiElement},
}, tilemap::tilemap_panel::TilemapPanel};

use engine_core::{
    assets::asset_manager::AssetManager,
    constants::*,
    ecs::{
        component::Position,
        entity::Entity,
        world_ecs::{WorldEcs},
    },
    tiles::{
        tile::{Tile, TileSprite},
        tilemap::TileMap,
    },
    world::{
        room::{Exit, ExitDirection, Room},
        world::GridPos,
    },
};

pub enum TilemapEditorMode {
    Tiles,
    Exits,
}

pub struct TileMapEditor {
    mode: TilemapEditorMode,
    dynamic_ui: Vec<Box<dyn DynamicTilemapUiElement>>,
    pub panel: TilemapPanel, 
    ui_was_clicked: bool,
    initialized: bool, 
}

impl TileMapEditor  {
    pub fn new() -> Self {
        let editor = Self {
            mode: TilemapEditorMode::Tiles,
            dynamic_ui: Vec::new(),
            panel: TilemapPanel::new(),
            ui_was_clicked: false,
            initialized: false,
        };

        editor
    }

    /// Update the editor with a mutable reference to the map
    pub async fn update(
        &mut self, 
        camera: &mut Camera2D,
        room: &mut Room,
        other_bounds: &[(Vec2, Vec2)],
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) 
        {
        if !self.initialized {
            self.ui_was_clicked = true; // Stop any initial tile placements
            self.initialized = true;
        }

        self.panel.update(world_ecs, asset_manager).await;

        self.dynamic_ui.clear();

        ResizeButton::build_all(&room.variants[0].tilemap, &mut self.dynamic_ui, room.position);
        
        let mouse_pos = mouse_position().into();
        self.consume_ui_click(camera, mouse_pos, room, other_bounds);

        if !self.ui_was_clicked {
            match self.mode {
                TilemapEditorMode::Tiles => self.handle_tile_placement(
                    camera, 
                    mouse_pos, 
                    &mut room.variants[0].tilemap, 
                    world_ecs, 
                    room.position
                ),
                TilemapEditorMode::Exits => self.handle_exit_placement(
                    camera, 
                    &room.variants[0].tilemap, 
                    &mut room.exits,
                    room.position
                ),
            }
        }

        if is_key_pressed(KeyCode::E) {
            self.toggle_exits();
        }
    }

    pub fn toggle_exits(&mut self) {
        self.mode = match self.mode {
            TilemapEditorMode::Exits => TilemapEditorMode::Tiles,
            _ => TilemapEditorMode::Exits,
        };
    }

    fn consume_ui_click(
        &mut self, 
        camera: &mut Camera2D,
        mouse_pos: Vec2,
        room: &mut Room,
        other_bounds: &[(Vec2, Vec2)]
    ) {
        if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right) {

            if self.panel.handle_click(mouse_pos, self.panel.rect) {
                self.ui_was_clicked = true;
                return;
            }

            for element in &mut self.dynamic_ui {
                if element.is_mouse_over(mouse_pos, camera) {
                    element.on_click(room, mouse_pos, camera, other_bounds);
                    self.ui_was_clicked = true;
                    break;
                }
            }
        }

        // Unblock UI
        if is_mouse_button_released(MouseButton::Left) || !is_mouse_button_down(MouseButton::Left) {
            self.ui_was_clicked = false;
        }
    }

    fn handle_tile_placement(
        &mut self, 
        camera: &Camera2D, 
        mouse_pos: Vec2, 
        map: &mut TileMap,
        world_ecs: &mut WorldEcs,
        room_position: Vec2,
    ) {
        let mouse_over_ui = self.is_mouse_over_ui(camera, mouse_pos);
        let hover = self.get_hovered_tile(camera, map, room_position);
        if mouse_over_ui || hover.is_none() { return; }

        let (x, y) = hover.unwrap().as_usize().unwrap();

        // Remove
        if is_mouse_button_down(MouseButton::Right) {
            let old = map.tiles[y][x];
            if old.entity != Entity::null() {
                world_ecs.remove_entity(old.entity);
            }
            map.tiles[y][x] = Tile::default();
            return;
        }

        let (def_id, sprite_id, sprite_path) = match (
            self.panel.palette.selected_def_opt(),
            self.panel.palette.selected_sprite_opt(),
            self.panel.palette.selected_path_opt(),
        ) {
            (Some(d), Some(s), Some(p)) => (d, s, p),
            _ => return, // There is no tile to place
        };

        // Place
        if is_mouse_button_down(MouseButton::Left) {
            // Grab the definition from the world
            let def = world_ecs
                .tile_defs
                .get(&def_id)
                .expect("definition must exist")
                .clone();

            // Build the base entity
            let mut builder = world_ecs
                .create_entity()
                .with(Position {
                    position: vec2(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ),
                })
                .with(TileSprite { 
                    sprite_id,
                    path: sprite_path.to_string(),
                });

            // Apply the behaviour definition (walkable, solid, damage, …)
            builder = def.apply(builder);

            // Finish and store the entity id in the grid cell
            let entity = builder.finish();
            map.tiles[y][x] = Tile { entity };
        }
    }

    fn handle_exit_placement(
        &mut self, 
        camera: &Camera2D, 
        map: &TileMap, 
        exits: &mut Vec<Exit>, 
        room_position: Vec2,
    ) {
        if let Some(tile_pos) = self.get_hovered_edge(camera, map, room_position) {
            let exit_direction = self.exit_direction_from_position(tile_pos, map);
            let exit_vec = vec2(tile_pos.x() as f32, tile_pos.y() as f32);

            if is_mouse_button_pressed(MouseButton::Left) {
                exits.push(Exit {
                    position: exit_vec,
                    direction: exit_direction,
                    target_room_id: None,
                });
            }

            if is_mouse_button_pressed(MouseButton::Right) {
                exits.retain(|exit| exit.position != exit_vec);
            }
        }
    }

    pub fn draw(
        &mut self, 
        camera: &Camera2D, 
        map: &mut TileMap, 
        exits: &Vec<Exit>,
        world_ecs: &WorldEcs,
        asset_manager: &mut AssetManager,
        room_position: Vec2,
    ) {
        clear_background(BLACK);
        set_camera(camera);
        map.draw(camera, exits, world_ecs, asset_manager, room_position);
        self.draw_hover_highlight(camera, map, room_position);
        self.draw_ui(camera, asset_manager, world_ecs, map);
    }

    fn draw_hover_highlight(&self, camera: &Camera2D, map: &TileMap, room_position: Vec2) {
        let tile_pos = match self.mode {
            TilemapEditorMode::Tiles => self.get_hovered_tile(camera, map, room_position),
            TilemapEditorMode::Exits => self.get_hovered_edge(camera, map, room_position),
        };

        if let Some(tile_pos) = tile_pos {
            let zoom_scale = camera.zoom.x.abs();
            let base_width = 0.5;
            let min_line_width = 2.0;
            let max_line_width = 5.0;
            let line_width = (base_width / zoom_scale).clamp(min_line_width, max_line_width);

            let x = tile_pos.x() as f32 * TILE_SIZE + room_position.x;
            let y = tile_pos.y() as f32 * TILE_SIZE + room_position.y;

            match self.mode {
                TilemapEditorMode::Tiles => {
                    draw_rectangle_lines(x, y, TILE_SIZE, TILE_SIZE, line_width, RED);
                }
                TilemapEditorMode::Exits => {
                    let exit_direction = self.exit_direction_from_position(tile_pos, map);
                    map.draw_exit(vec2(x, y), exit_direction);
                }
            }
        }
    }

    fn draw_ui(
        &mut self, 
        camera: &Camera2D, 
        asset_manager: &mut AssetManager,
        world_ecs: &WorldEcs,
        map: &mut TileMap,
    ) {
        // Draw scaling UI
        for element in &self.dynamic_ui {
            element.draw(camera);
        }
        
        // Reset to default camera for static UI drawing
        set_default_camera();

        // Draw panel
        self.panel.draw(asset_manager, world_ecs, map);
    }

    fn get_hovered_tile(&self, camera: &Camera2D, map: &TileMap, room_position: Vec2) -> Option<GridPos> {
        let mouse_pos: Vec2 = mouse_position().into();
        let world_pos = camera.screen_to_world(mouse_pos);
        let local_pos = world_pos - room_position;
        let pos = GridPos::from_world(local_pos);

        if pos.is_in_bounds(map.width, map.height) {
            Some(pos)
        } else {
            None
        }
    }

    fn get_hovered_edge(&self, camera: &Camera2D, map: &TileMap, room_position: Vec2) -> Option<GridPos> {
        let mouse_pos: Vec2 = mouse_position().into();
        let world_pos = camera.screen_to_world(mouse_pos);
        let local_pos = world_pos - room_position;
        let edge_pos = GridPos::from_world_edge(local_pos, map);

        let x_outside = edge_pos.x() < 0 || edge_pos.x() >= map.width as i32;
        let y_outside = edge_pos.y() < 0 || edge_pos.y() >= map.height as i32;

        // Only allow positions strictly outside one axis (no corners)
        if x_outside ^ y_outside {
            Some(edge_pos)
        } else {
            None
        }
    }

    fn is_mouse_over_ui(&self, camera: &Camera2D, mouse_pos: Vec2) -> bool {
        self.panel.is_mouse_over(mouse_pos)
        || self.dynamic_ui
            .iter()
            .any(|element| element.is_mouse_over(mouse_pos, camera))
    }

    fn exit_direction_from_position(&self, tile_pos: GridPos, map: &TileMap) -> ExitDirection {
        match tile_pos {
            GridPos(p) if p.y == -1 => ExitDirection::Up,
            GridPos(p) if p.y == map.height as i32 => ExitDirection::Down,
            GridPos(p) if p.x == -1 => ExitDirection::Left,
            GridPos(p) if p.x == map.width as i32 => ExitDirection::Right,
            GridPos(p) if p.y == 0 => ExitDirection::Up,
            GridPos(p) if p.y as usize == map.height - 1 => ExitDirection::Down,
            GridPos(p) if p.x == 0 => ExitDirection::Left,
            GridPos(p) if p.x as usize == map.width - 1 => ExitDirection::Right,
            _ => ExitDirection::Up, // default for safety
        }
    }

    pub fn reset(&mut self) {
        self.mode = TilemapEditorMode::Tiles;
        self.initialized = false;
        self.ui_was_clicked = false;
    }
}