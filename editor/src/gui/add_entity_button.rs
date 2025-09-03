
use macroquad::prelude::*;
use core::{ecs::{
    component::{Position, Velocity}, world_ecs::WorldEcs
}, world::room::RoomMetadata};

pub struct AddEntityButton {
    rect: Rect,
}

impl AddEntityButton {
    pub fn new() -> Self {
        let width = 120.0;
        let height = 30.0;
        let x = 20.0;
        let y = 20.0;
        Self {
            rect: Rect::new(x, y, width, height),
        }
    }

    /// Draw the button (screen‑space).  Hover = light‑gray, normal = gray.
    pub fn draw(&self) {
        let hovered = mouse_over_rect(self.rect);
        let bg = if hovered { LIGHTGRAY } else { GRAY };
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, bg);
        draw_rectangle_lines(self.rect.x, self.rect.y, self.rect.w, self.rect.h, 2.0, BLACK);
        draw_text(
            "Add Entity",
            self.rect.x + 10.0,
            self.rect.y + self.rect.h * 0.65,
            24.0,
            if hovered { RED } else { BLACK },
        );
    }

    /// Create a fresh entity whose world‑position is the origin of the current room.
    pub fn try_click(&self, ecs: &mut WorldEcs, room_meta: &RoomMetadata) {
        if mouse_over_rect(self.rect) && is_mouse_button_pressed(MouseButton::Left) {
            let start_pos = room_meta.position;
            ecs.create_entity()
                .with(Position {
                    position: start_pos,
                })
                .with(Velocity {
                    vel: Vec2::ZERO,
                })
                .finish();
            println!("{}", start_pos)
        }
    }
}

fn mouse_over_rect(rect: Rect) -> bool {
    let (mx, my) = mouse_position();
    rect.contains(vec2(mx, my))
}