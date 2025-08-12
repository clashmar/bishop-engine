use core::{constants::TILE_SIZE, tile::Tile, tilemap::TileMap};
use macroquad::prelude::*;
use crate::gui::{text_button::TextButton, ui_element::UiElement};

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

impl UiElement for ResizeButton {
    fn draw(&self, camera: &Camera2D) {
        let mouse_world_pos = camera.screen_to_world(mouse_position().into());
        let hovered = self.button.is_hovered(mouse_world_pos);
        self.button.draw(hovered);
    }

    fn is_mouse_over(&self, mouse_pos: Vec2, camera: &Camera2D) -> bool {
        let world_pos = camera.screen_to_world(mouse_pos);
        self.button.is_hovered(world_pos)
    }

    fn on_click(&mut self, map: &mut TileMap, _selected_tile: &mut Tile, mouse_pos: Vec2, camera: &Camera2D) {
        let mouse_world_pos = camera.screen_to_world(mouse_pos);
        if self.button.is_clicked(mouse_world_pos) {
            // Perform the resize action immediately here
            match self.action {
                ResizeAction::AddTop => {
                    map.tiles.insert(0, vec![Tile::none(); map.width]);
                    map.height += 1;
                }
                ResizeAction::RemoveTop => {
                    if map.height > 1 {
                        map.tiles.remove(0);
                        map.height -= 1;
                    }
                }
                ResizeAction::AddBottom => {
                    map.tiles.push(vec![Tile::none(); map.width]);
                    map.height += 1;
                }
                ResizeAction::RemoveBottom => {
                    if map.height > 1 {
                        map.tiles.pop();
                        map.height -= 1;
                    }
                }
                ResizeAction::AddLeft => {
                    for row in &mut map.tiles {
                        row.insert(0, Tile::none());
                    }
                    map.width += 1;
                }
                ResizeAction::RemoveLeft => {
                    if map.width > 1 {
                        for row in &mut map.tiles {
                            row.remove(0);
                        }
                        map.width -= 1;
                    }
                }
                ResizeAction::AddRight => {
                    for row in &mut map.tiles {
                        row.push(Tile::none());
                    }
                    map.width += 1;
                }
                ResizeAction::RemoveRight => {
                    if map.width > 1 {
                        for row in &mut map.tiles {
                            row.pop();
                        }
                        map.width -= 1;
                    }
                }
            }
        }
    }
}

impl ResizeButton {
    pub fn build_all(map: &TileMap, ui_elements: &mut Vec<Box<dyn UiElement>>) {
        let margin = TILE_SIZE / 4.0;
        let btn_size = vec2(30.0, 30.0);

        let map_pixel_width = map.width as f32 * TILE_SIZE;
        let map_pixel_height = map.height as f32 * TILE_SIZE;

        let mut add_btn = |action: ResizeAction, world_pos: Vec2, label: &str, color: Color| {
            let rect = Rect::new(
                world_pos.x - btn_size.x / 2.0,
                world_pos.y - btn_size.y / 2.0,
                btn_size.x,
                btn_size.y,
            );
            let btn = TextButton {
                rect,
                label: label.to_string(),
                background_color: color,
                text_color: BLACK,
                font_size: 50.0,
            };
            ui_elements.push(Box::new(ResizeButton { action, button: btn }));
        };

        add_btn(
            ResizeAction::AddTop,
            vec2(map_pixel_width / 2.0, map_pixel_height + margin + 60.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveTop,
            vec2(map_pixel_width / 2.0, map_pixel_height + margin + 20.0),
            "-",
            RED,
        );

        add_btn(
            ResizeAction::RemoveBottom,
            vec2(map_pixel_width / 2.0, -margin - 20.0),
            "-",
            RED,
        );
        add_btn(
            ResizeAction::AddBottom,
            vec2(map_pixel_width / 2.0, -margin - 60.0),
            "+",
            GREEN,
        );

        add_btn(
            ResizeAction::AddLeft,
            vec2(-margin - 60.0, map_pixel_height / 2.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveLeft,
            vec2(-margin - 20.0, map_pixel_height / 2.0),
            "-",
            RED,
        );

        add_btn(
            ResizeAction::AddRight,
            vec2(map_pixel_width + margin + 60.0, map_pixel_height / 2.0),
            "+",
            GREEN,
        );
        add_btn(
            ResizeAction::RemoveRight,
            vec2(map_pixel_width + margin + 20.0, map_pixel_height / 2.0),
            "-",
            RED,
        );
    }
}