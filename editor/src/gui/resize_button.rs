// editor/src/gui/resize_button.rs
use crate::gui::ui_element::DynamicTilemapUiElement;
use crate::gui::text_button::TextButton;
use crate::engine_global::tile_size;
use crate::tiles::tilemap::*;
use crate::ecs::ecs::Ecs;
use crate::world::coord;
use engine_core::world::room::Room;
use macroquad::prelude::*;

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
        _world_ecs: &mut Ecs,
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

        delta_pos *= tile_size();
        proposed_size *= tile_size();

        // Check for overlaps
        let proposed_pos = *room_position + delta_pos;

        if coord::overlaps_existing_rooms(proposed_pos, proposed_size, other_bounds) {
            // Skip the resize
            return;
        }

        // Apply resize
        match self.action {
            ResizeAction::AddTop => {
                map.height += 1;
                shift_tiles(map, 0, 1);
                for exit in &mut room.exits {
                    let exit_grid_y = room_size.y - exit.position.y; 
                    if (exit_grid_y - 0.0).abs() < f32::EPSILON {
                        // exit is on the top row
                        exit.position.y += 1.0; // move up in exit-space
                    }
                }

                room_size.y += 1.0;
                room_position.y -= 1.0 * tile_size();
                
            }
            ResizeAction::RemoveTop => {
                if map.height > 1 {
                    for x in 0..map.width {
                        map.remove_tile((x, 0));
                    }
                    map.height -= 1;
                    shift_tiles(map, 0, -1);

                    for exit in &mut room.exits {
                        let exit_grid_y = room_size.y - exit.position.y; // convert exit y to grid y
                        if (exit_grid_y - 0.0).abs() < f32::EPSILON {
                            // exit is on the top row, which is being removed
                            exit.position.y -= 1.0; // move down in exit-space
                        }
                    }

                    room_size.y -= 1.0;
                    room_position.y += 1.0 * tile_size();
                }
            }
            ResizeAction::AddBottom => {
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
                    let bottom = map.height - 1;
                    for x in 0..map.width {
                        map.remove_tile((x, bottom));
                    }
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
                map.width += 1;
                shift_tiles(map, 1, 0);
                room_size.x += 1.0;
                room_position.x -= 1.0 * tile_size();
            }
            ResizeAction::RemoveLeft => {
                if map.width > 1 {
                    for y in 0..map.height {
                        map.remove_tile((0, y));
                    }
                    map.width -= 1;
                    shift_tiles(map, -1, 0);
                    room_size.x -= 1.0;
                    room_position.x += 1.0 * tile_size();
                }
            }
            ResizeAction::AddRight => {
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
                    let right = map.width - 1;
                    for y in 0..map.height {
                        map.remove_tile((right, y));
                    }
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
        let margin: f32 = tile_size() / 8.0;
        let btn_size: Vec2 = vec2(tile_size() / 1.75, tile_size() / 1.75);
        let outer_gap: f32 = tile_size() * 3.;
        let inner_gap: f32 = tile_size() * 2.;

        let map_pixel_width = map.width as f32 * tile_size();
        let map_pixel_height = map.height as f32 * tile_size();

        let mut add_btn = |action: ResizeAction, local_position: Vec2, label: &str, color: Color| {
            let rect = Rect::new(
                (local_position.x + room_position.x) - btn_size.x / 2.0,
                (local_position.y + room_position.y) - btn_size.y / 2.0,
                btn_size.x,
                btn_size.y,
            );
            let btn = TextButton {
                rect,
                label: label.to_string(),
                background_color: color,
                text_color: BLACK,
                font_size: tile_size(),
            };
            ui_elements.push(Box::new(ResizeButton { action, button: btn }));
        };

        add_btn(
            ResizeAction::AddTop,
            vec2(map_pixel_width / 2.0, -margin - outer_gap),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveTop,
            vec2(map_pixel_width / 2.0, -margin - inner_gap),
            "-",
            RED,
        );

        add_btn(
            ResizeAction::RemoveBottom,
            vec2(map_pixel_width / 2.0, map_pixel_height + margin + inner_gap),
            "-",
            RED,
        );
        add_btn(
            ResizeAction::AddBottom,
            vec2(map_pixel_width / 2.0, map_pixel_height + margin + outer_gap),
            "+",
            GREEN,
        );

        add_btn(
            ResizeAction::AddLeft,
            vec2(-margin - outer_gap, map_pixel_height / 2.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveLeft,
            vec2(-margin - inner_gap, map_pixel_height / 2.0),
            "-",
            RED,
        );

        add_btn(
            ResizeAction::AddRight,
            vec2(map_pixel_width + margin + outer_gap, map_pixel_height / 2.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveRight,
            vec2(map_pixel_width + margin + inner_gap, map_pixel_height / 2.0),
            "-",
            RED,
        );
    }
}