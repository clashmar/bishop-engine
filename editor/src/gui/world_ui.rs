use core::{constants::WORLD_SAVE_FOLDER, world::world::World};
use std::{future::Future, path::Path, pin::Pin};

use crate::{gui::ui_element::WorldUiElement, storage::world_storage, world::world_editor::mouse_over_rect};
use macroquad::prelude::*;

const FONT_SIZE: f32 = 40.0;
const HORIZONTAL_PADDING: f32 = 20.0;
const VERTICAL_OFFSET: f32 = 10.0; 

pub struct WorldNameUi;

impl WorldNameUi {
    pub fn new() -> Self {
        WorldNameUi {}
    }
}

impl WorldUiElement for WorldNameUi {
    fn rect(&self, world: &World) -> Option<Rect> {
        let text_width = measure_text(&world.name, None, FONT_SIZE as u16, 1.0).width;
        let box_width = text_width + HORIZONTAL_PADDING * 2.0;
        let rect_height = FONT_SIZE * 1.2;
        let x = (screen_width() - box_width) / 2.0;
        let y = 40.0 + VERTICAL_OFFSET;
        Some(Rect::new(x, y - FONT_SIZE, box_width, rect_height))
    }

    fn on_click<'a>(&'a self, world: &'a mut World) -> Pin<Box<dyn Future<Output=()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(new_name) = world_storage::prompt_user_input().await {
                let new_name = new_name.trim().to_string();
                let old_name = world.name.clone();
                if new_name.trim().is_empty() || new_name == old_name { return; }

                println!("{}.", &new_name);

                let new_path = format!("{}/{}.ron", WORLD_SAVE_FOLDER, &new_name);
                if Path::new(&new_path).exists() {
                    eprintln!("World with name '{}' already exists, aborting rename.", new_name);
                    return;
                }

                 let old_path = format!("{}/{}.ron", WORLD_SAVE_FOLDER, old_name);

                if Path::new(&old_path).exists() {
                    let new_path = format!("{}/{}.ron", WORLD_SAVE_FOLDER, &new_name);
                    if let Err(err) = std::fs::rename(&old_path, &new_path) {
                        eprintln!("Failed to rename world file: {}", err);
                    }
                }

                world.name = new_name;
                world_storage::save_world(world).expect("Could not save world.");
            }
        })
    }

    fn draw(&self, world: &World) {
        set_default_camera(); // screen space

        let text_width = measure_text(&world.name, None, FONT_SIZE as u16, 1.0).width;
        let box_width = text_width + HORIZONTAL_PADDING * 2.0;
        let rect_height = FONT_SIZE * 1.2;
        
        let x = (screen_width() - box_width) / 2.0;
        let y = 40.0 + VERTICAL_OFFSET; // tweak to move box/text up/down

        let rect = Rect::new(x, y - FONT_SIZE, box_width, FONT_SIZE * 1.2);
        let is_hovered = mouse_over_rect(rect);
        let text_color = if is_hovered { RED } else { BLACK };
        
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 8.0, BLACK);
        let text_y = y + rect_height / 2.0 + FONT_SIZE / 2.0 * -1.5; // tweak to move text up/down
        draw_text(&world.name, x + HORIZONTAL_PADDING * 1.5, text_y, FONT_SIZE, text_color);
    }
}