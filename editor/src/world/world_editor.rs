use core::world::{room::Room, world::World};
use macroquad::prelude::*;

const ROOM_SCALE_FACTOR: f32 = 9.0;
const WORLD_EDITOR_ZOOM_FACTOR: f32 = 1.0;

pub struct WorldEditor {
    pub world: World,
    camera: Camera2D,
}

impl WorldEditor {
    pub fn new(width: usize, height: usize) -> Self {
        let mut world = World::new();

        let first_room_idx = world.create_room(
            "Room1",
            vec2(0.0, 0.0),
            vec2(width as f32, height as f32)
        );

        let camera = Self::compute_camera_for_room(&world.rooms[first_room_idx]);

        Self { world, camera }
    }

    /// Returns `true` if a room is clicked on.
    pub fn update(&mut self) -> Option<usize> {
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = vec2(mouse_position().0, mouse_position().1);
            let world_mouse = self.camera.screen_to_world(mouse_pos);

            for (i, room) in self.world.rooms.iter().enumerate() {
                let room_size = room.size();
                let rect = Rect::new(
                    room.position.x * ROOM_SCALE_FACTOR,
                    room.position.y * ROOM_SCALE_FACTOR,
                    room_size.x * ROOM_SCALE_FACTOR,
                    room_size.y * ROOM_SCALE_FACTOR,
                );
                if rect.contains(world_mouse) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn draw(&self) {
        set_camera(&self.camera);
        clear_background(LIGHTGRAY);

        for room in &self.world.rooms {
            let room_size = room.size();
            let scaled_x = room.position.x * ROOM_SCALE_FACTOR;
            let scaled_y = room.position.y * ROOM_SCALE_FACTOR;
            let scaled_w = room_size.x * ROOM_SCALE_FACTOR;
            let scaled_h = room_size.y * ROOM_SCALE_FACTOR;

            // Draw room outline
            draw_rectangle_lines(
                scaled_x,
                scaled_y,
                scaled_w,
                scaled_h,
                2.0,
                BLUE,
            );

            // Draw text centered inside the room
            let text_size = measure_text(&room.name, None, 20, 1.0);
            let text_x = scaled_x + (scaled_w - text_size.width) / 2.0;
            let text_y = scaled_y + (scaled_h + text_size.height) / 2.0; // y is baseline
            draw_text(
                &room.name,
                text_x,
                text_y,
                20.0,
                BLACK,
            );
        }

        set_default_camera();
    }

     pub fn center_on_room(&mut self, room_idx: usize) {
        let room = &self.world.rooms[room_idx];
        self.camera = Self::compute_camera_for_room(room);
    }

    fn compute_camera_for_room(room: &Room) -> Camera2D {
        let room_size = room.size();
        let room_scaled_size = room_size * ROOM_SCALE_FACTOR;

        let zoom_x = WORLD_EDITOR_ZOOM_FACTOR / room_scaled_size.x;
        let zoom_y = WORLD_EDITOR_ZOOM_FACTOR / room_scaled_size.y;

        Camera2D {
            target: (room.position + room_size / 2.0) * ROOM_SCALE_FACTOR,
            zoom: vec2(zoom_x, zoom_y),
            ..Default::default()
        }
    }
}