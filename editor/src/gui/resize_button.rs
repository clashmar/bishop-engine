// editor/src/gui/resize_button.rs
use macroquad::prelude::*;
use engine_core::{constants::TILE_SIZE, tiles::{tile::Tile, tilemap::TileMap}, world::room::Room};
use crate::{gui::{text_button::TextButton, ui_element::DynamicTilemapUiElement}, world::coord};

pub struct ResizeButton {
    pub action: ResizeAction,
    pub button: TextButton,
}

pub enum ResizeAction {
    AddTop,
    RemoveTop,
    AddBottom,
    RemoveBottom,
    AddLeft,
    RemoveLeft,
    AddRight,
    RemoveRight,
}

impl DynamicTilemapUiElement for ResizeButton {
    fn draw(&self, camera: &Camera2D) {
        let mouse_world_pos = camera.screen_to_world(mouse_position().into());
        let hovered = self.button.is_hovered(mouse_world_pos);
        self.button.draw(hovered);
    }

    fn is_mouse_over(&self, mouse_pos: Vec2, camera: &Camera2D) -> bool {
        let world_pos = camera.screen_to_world(mouse_pos);
        self.button.is_hovered(world_pos)
    }

    fn on_click(
        &mut self,
        room: &mut Room,
        mouse_pos: Vec2, 
        camera: &Camera2D,
        other_bounds: &[(Vec2, Vec2)],
    ) {
        let mouse_world_pos = camera.screen_to_world(mouse_pos);
        if !self.button.is_clicked(mouse_world_pos) {
            return;
        }

        let room_position = &mut room.position;
        let room_size = &mut room.size;
        let map = &mut room.variants[0].tilemap;

        // Compute proposed delta and new size
        let (mut delta_pos, mut proposed_size) = match self.action {
            ResizeAction::AddTop    => (vec2(0.0, -1.0), vec2(map.width as f32, map.height as f32 + 1.0)),
            ResizeAction::RemoveTop => (vec2(0.0,  1.0), vec2(map.width as f32, map.height as f32 - 1.0)),
            ResizeAction::AddBottom    => (vec2(0.0, 0.0), vec2(map.width as f32, map.height as f32 + 1.0)),
            ResizeAction::RemoveBottom => (vec2(0.0, 0.0), vec2(map.width as f32, map.height as f32 - 1.0)),
            ResizeAction::AddLeft   => (vec2(-1.0, 0.0), vec2(map.width as f32 + 1.0, map.height as f32)),
            ResizeAction::RemoveLeft=> (vec2( 1.0, 0.0), vec2(map.width as f32 - 1.0, map.height as f32)),
            ResizeAction::AddRight  => (vec2(0.0, 0.0), vec2(map.width as f32 + 1.0, map.height as f32)),
            ResizeAction::RemoveRight=> (vec2(0.0, 0.0), vec2(map.width as f32 - 1.0, map.height as f32)),
        };

        delta_pos *= TILE_SIZE;
        proposed_size *= TILE_SIZE;

        // Check for overlaps
        let proposed_pos = *room_position + delta_pos;

        if coord::overlaps_existing_rooms(proposed_pos, proposed_size, other_bounds) {
            // Skip the resize
            return;
        }

        // Apply resize
        match self.action {
            ResizeAction::AddTop => {
                map.tiles.insert(0, vec![Tile::default(); map.width]);
                map.height += 1;

                for exit in &mut room.exits {
                    let exit_grid_y = room_size.y - exit.position.y; 
                    if (exit_grid_y - 0.0).abs() < f32::EPSILON {
                        // exit is on the top row
                        exit.position.y += 1.0; // move up in exit-space
                    }
                }

                room_size.y += 1.0;
                room_position.y -= 1.0 * TILE_SIZE;
                
            }
            ResizeAction::RemoveTop => {
                if map.height > 1 {
                    map.tiles.remove(0);
                    map.height -= 1;

                    for exit in &mut room.exits {
                        let exit_grid_y = room_size.y - exit.position.y; // convert exit y to grid y
                        if (exit_grid_y - 0.0).abs() < f32::EPSILON {
                            // exit is on the top row, which is being removed
                            exit.position.y -= 1.0; // move down in exit-space
                        }
                    }

                    room_size.y -= 1.0;
                    room_position.y += 1.0 * TILE_SIZE;
                }
            }
            ResizeAction::AddBottom => {
                map.tiles.push(vec![Tile::default(); map.width]);
                map.height += 1;
                for exit in &mut room.exits {
                    if (exit.position.y - room_size.y).abs() < f32::EPSILON {
                        // the exit sits on the bottom edge → shift it down
                        exit.position.y += 1.0;
                    }
                }
                room_size.y += 1.0;
            }
            ResizeAction::RemoveBottom => {
                if map.height > 1 {
                    map.tiles.pop();
                    map.height -= 1;
                    for exit in &mut room.exits {
                        if (exit.position.y - room_size.y).abs() < f32::EPSILON {
                            // the exit sits on the bottom edge → shift it up
                            exit.position.y -= 1.0;
                        }
                    }
                    room_size.y -= 1.0;
                }
            }
            ResizeAction::AddLeft => {
                for row in &mut map.tiles { row.insert(0, Tile::default()); }
                map.width += 1;
                room_size.x += 1.0;
                room_position.x -= 1.0 * TILE_SIZE;
            }
            ResizeAction::RemoveLeft => {
                if map.width > 1 {
                    for row in &mut map.tiles { row.remove(0); }
                    map.width -= 1;
                    room_size.x -= 1.0;
                    room_position.x += 1.0 * TILE_SIZE;
                }
            }
            ResizeAction::AddRight => {
                for row in &mut map.tiles { row.push(Tile::default()); }
                map.width += 1;

                for exit in &mut room.exits {
                    // exit-space x increases to the right, so if it's on the right edge, shift it
                    if (exit.position.x - room_size.x).abs() < f32::EPSILON {
                        exit.position.x += 1.0;
                    }
                }

                room_size.x += 1.0;
            }
            ResizeAction::RemoveRight => {
                if map.width > 1 {
                    for row in &mut map.tiles { row.pop(); }
                    map.width -= 1;

                    for exit in &mut room.exits {
                        // if exit was on the rightmost column, move it left
                        if (exit.position.x - room_size.x).abs() < f32::EPSILON {
                            exit.position.x -= 1.0;
                        }
                    }
                    
                    room_size.x -= 1.0;
                }
            }
        }
    }
}

impl ResizeButton {
    pub fn build_all(
        map: &TileMap, 
        ui_elements: &mut Vec<Box<dyn DynamicTilemapUiElement>>,
        room_position: Vec2,
    ) {
        const MARGIN: f32 = TILE_SIZE / 8.0;
        const BTN_SIZE: Vec2 = vec2(15.0, 15.0);
        const OUTER_GAP: f32 = 50.0;
        const INNER_GAP: f32 = 30.0;

        let map_pixel_width = map.width as f32 * TILE_SIZE;
        let map_pixel_height = map.height as f32 * TILE_SIZE;

        let mut add_btn = |action: ResizeAction, local_position: Vec2, label: &str, color: Color| {
            let rect = Rect::new(
                (local_position.x + room_position.x) - BTN_SIZE.x / 2.0,
                (local_position.y + room_position.y) - BTN_SIZE.y / 2.0,
                BTN_SIZE.x,
                BTN_SIZE.y,
            );
            let btn = TextButton {
                rect,
                label: label.to_string(),
                background_color: color,
                text_color: BLACK,
                font_size: 25.0,
            };
            ui_elements.push(Box::new(ResizeButton { action, button: btn }));
        };

        add_btn(
            ResizeAction::AddTop,
            vec2(map_pixel_width / 2.0, -MARGIN - OUTER_GAP),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveTop,
            vec2(map_pixel_width / 2.0, -MARGIN - INNER_GAP),
            "-",
            RED,
        );

        add_btn(
            ResizeAction::RemoveBottom,
            vec2(map_pixel_width / 2.0, map_pixel_height + MARGIN + INNER_GAP),
            "-",
            RED,
        );
        add_btn(
            ResizeAction::AddBottom,
            vec2(map_pixel_width / 2.0, map_pixel_height + MARGIN + OUTER_GAP),
            "+",
            GREEN,
        );

        add_btn(
            ResizeAction::AddLeft,
            vec2(-MARGIN - OUTER_GAP, map_pixel_height / 2.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveLeft,
            vec2(-MARGIN - INNER_GAP, map_pixel_height / 2.0),
            "-",
            RED,
        );

        add_btn(
            ResizeAction::AddRight,
            vec2(map_pixel_width + MARGIN + OUTER_GAP, map_pixel_height / 2.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveRight,
            vec2(map_pixel_width + MARGIN + INNER_GAP, map_pixel_height / 2.0),
            "-",
            RED,
        );
    }
}