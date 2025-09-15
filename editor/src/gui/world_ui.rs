use engine_core::{world::world::World};
use std::{future::Future, pin::Pin};

use crate::{gui::ui_element::WorldUiElement, storage::editor_storage, world::world_editor::mouse_over_rect};
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
            if let Some(new_name) = editor_storage::prompt_user_input().await {
                let new_name = new_name.trim().to_string();
                if new_name.is_empty() || new_name == world.name { return; }

                // Update the index
                let mut idx = editor_storage::load_index().expect("load index");
                idx.insert(world.id, new_name.clone());
                editor_storage::save_index(&idx).expect("save index");

                // Update the in‑memory struct and persist the single file
                world.name = new_name;
                editor_storage::save_world(world).expect("save world");
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