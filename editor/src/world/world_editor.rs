use core::world::{room::{ExitDirection, Room}, world::World};
use macroquad::prelude::*;

const ROOM_SCALE_FACTOR: f32 = 8.0;
const WORLD_EDITOR_ZOOM_FACTOR: f32 = 1.0;
const HIGHLIGHT_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.5);
const HIGHLIGHT_ERROR_COLOR: Color = Color::new(1.0, 0.0, 0.0, 0.5);
const LINE_THICKNESS_MULTIPLIER: f32 = 0.02;
const ROOM_LINE_INSET: f32 = 0.5;
const GRID_LINE_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.2);
const HOVER_LINE_THICKNESS: f32 = 0.05;
const PAN_SPEED: f32 = 500.0;
const ZOOM_SPEED_FACTOR: f32 = 0.1;
const MIN_ZOOM: f32 = 0.003;
const MAX_ZOOM: f32 = 0.01;

pub enum WorldEditorMode {
    Selecting,
    PlacingRoom,
    DeletingRoom,
}

pub struct WorldEditor {
    pub world: World,
    camera: Camera2D,
    mode: WorldEditorMode,
    show_grid: bool,
    placing_start: Option<Vec2>,
    placing_end: Option<Vec2>, 
}

impl WorldEditor {
    pub fn new(width: usize, height: usize) -> Self {
        let mut world = World::new();

        let first_room_idx = world.create_room(
            "1",
            vec2(0.0, 0.0),
            vec2(width as f32, height as f32)
        );

        let camera = Self::compute_camera_for_room(&world.rooms[first_room_idx]);

        Self { 
            world, 
            camera, 
            mode: WorldEditorMode::Selecting,
            show_grid: true,
            placing_start: None,
            placing_end: None,
        }
    }

    /// Returns `true` if a room is clicked on.
    pub fn update(&mut self) -> Option<usize> {
        let dt = get_frame_time();
        self.update_camera(dt);

        self.world.link_all_exits();

        if is_key_pressed(KeyCode::C) {
            self.toggle_placing_room();
        }

        if is_key_pressed(KeyCode::X) {
            self.toggle_delete_room();
        }

        if is_key_pressed(KeyCode::G) {
            self.show_grid = !self.show_grid;
        }

        match self.mode {
            WorldEditorMode::Selecting => self.update_selecting_mode(),
            WorldEditorMode::PlacingRoom => self.update_placing_mode(),
            WorldEditorMode::DeletingRoom => self.update_deleting_mode(),
        }
    }

    fn update_selecting_mode(&mut self) -> Option<usize> {
        if is_mouse_button_pressed(MouseButton::Left) {
            let world_mouse = self.mouse_world_pos();
            for (i, room) in self.world.rooms.iter().enumerate() {
                let rect = scaled_room_rect(room);
                if rect.contains(world_mouse) {
                    return Some(i);
                }
            }
        }
        None
    }

    fn update_deleting_mode(&mut self) -> Option<usize> {
        if is_mouse_button_pressed(MouseButton::Left) {
            let world_mouse = self.mouse_world_pos();
            for (i, room) in self.world.rooms.iter().enumerate() {
                let rect = scaled_room_rect(room);
                if rect.contains(world_mouse) {
                    if let WorldEditorMode::DeletingRoom = self.mode {
                        self.world.delete_room(i);
                        return None;
                    }
                }
            }
        }
        None
    }

    fn update_placing_mode(&mut self) -> Option<usize> {
        let mouse_tile = self.snap_to_grid(self.mouse_world_pos() / ROOM_SCALE_FACTOR);

        if is_mouse_button_pressed(MouseButton::Left) {
            self.placing_start = Some(mouse_tile);
            self.placing_end = Some(mouse_tile);
        }

        if is_mouse_button_down(MouseButton::Left) {
            self.placing_end = Some(mouse_tile);
        }

        if is_mouse_button_released(MouseButton::Left) {
            if let (Some(start), Some(end)) = (self.placing_start, self.placing_end) {
                let (top_left, size) = rect_from_points(start, end);

                if !self.intersects_existing_room(top_left, size) {
                    let room_idx = self.world.create_room("NewRoom", top_left, size);
                    self.reset_placing();
                    self.mode = WorldEditorMode::Selecting;
                    return Some(room_idx);
                }
                self.reset_placing();
            }
        }

        None
    }

    fn intersects_existing_room(&self, top_left: Vec2, size: Vec2) -> bool {
        let a_left = top_left.x;
        let a_right = top_left.x + size.x;
        let a_top = top_left.y;
        let a_bottom = top_left.y + size.y;

        for room in &self.world.rooms {
            let b_left = room.position.x;
            let b_right = room.position.x + room.size().x;
            let b_top = room.position.y;
            let b_bottom = room.position.y + room.size().y;

            // Return true only if the rectangles actually overlap
            let intersects = a_left < b_right && a_right > b_left &&
                            a_top < b_bottom && a_bottom > b_top;

            if intersects {
                return true;
            }
        }
        false
    }

    fn reset_placing(&mut self) {
        self.placing_start = None;
        self.placing_end = None;
    }

    pub fn toggle_placing_room(&mut self) {
        self.mode = match self.mode {
            WorldEditorMode::PlacingRoom => WorldEditorMode::Selecting,
            _ => WorldEditorMode::PlacingRoom,
        };
    }

    pub fn toggle_delete_room(&mut self) {
        self.mode = match self.mode {
            WorldEditorMode::DeletingRoom => WorldEditorMode::Selecting,
            _ => WorldEditorMode::DeletingRoom,
        };
    }

    fn snap_to_grid(&self, pos: Vec2) -> Vec2 {
        vec2(
            (pos.x).floor(),
            (pos.y).floor(),
        )
    }

    fn mouse_world_pos(&self) -> Vec2 {
        let (x, y) = mouse_position();
        self.camera.screen_to_world(vec2(x, y))
    }

    pub fn draw(&self) {
        set_camera(&self.camera);
        clear_background(LIGHTGRAY);

        if self.show_grid {
            self.draw_grid();
        }

        self.draw_rooms();
        self.draw_unlinked_exits();

        // Highlight hovered room in select or delete mode
        match self.mode {
            WorldEditorMode::Selecting | 
            WorldEditorMode::DeletingRoom => self.draw_hovered_room(),
            _ => {},
        }

        if let WorldEditorMode::PlacingRoom = self.mode {
            self.draw_placing_preview();
        }

        self.draw_room_names(); 

        set_default_camera();
    }

    pub fn draw_rooms(&self) {
        for room in &self.world.rooms {
            let rect = scaled_room_rect(room);
            let inset = ROOM_LINE_INSET * ROOM_SCALE_FACTOR;

            // Draw the room outline
            draw_rectangle_lines(
                rect.x + inset / 2.0,
                rect.y + inset / 2.0,
                rect.w - inset,
                rect.h - inset,
                LINE_THICKNESS_MULTIPLIER / self.camera.zoom.x,
                BLUE,
            );
        }
    }

    fn draw_unlinked_exits(&self) {
    for room in &self.world.rooms {
        for (exit_world_pos, dir) in room.world_exit_positions() {
            for exit in &room.exits {
                let pos = room.position + Vec2::new(
                    exit.position.x,
                    room.size().y - exit.position.y - 1.0
                );

                if (pos - exit_world_pos).length_squared() < 0.01 {
                    // Decide color based on whether it's linked
                    let color = if exit.target_room_id.is_some() {
                        GREEN
                    } else {
                        RED
                    };
                    self.draw_exit_marker(exit_world_pos, dir, color);
                }
            }
        }
    }
}

    fn draw_exit_marker(&self, exit_world_pos: Vec2, dir: ExitDirection, color: Color) {
    let thickness = 2.0;
    let length = ROOM_SCALE_FACTOR;
    let offset = 1.0; 

    match dir {
        ExitDirection::Up => draw_rectangle(
            exit_world_pos.x * ROOM_SCALE_FACTOR,
            exit_world_pos.y * ROOM_SCALE_FACTOR - offset,
            length,
            thickness,
            color,
        ),
        ExitDirection::Down => draw_rectangle(
            exit_world_pos.x * ROOM_SCALE_FACTOR,
            exit_world_pos.y * ROOM_SCALE_FACTOR + ROOM_SCALE_FACTOR - thickness + offset,
            length,
            thickness,
            color,
        ),
        ExitDirection::Left => draw_rectangle(
            (exit_world_pos.x + 1.0) * ROOM_SCALE_FACTOR - offset,
            exit_world_pos.y * ROOM_SCALE_FACTOR,
            thickness,
            length,
            color,
        ),
        ExitDirection::Right => draw_rectangle(
            (exit_world_pos.x - 1.0) * ROOM_SCALE_FACTOR + ROOM_SCALE_FACTOR - thickness + offset,
            exit_world_pos.y * ROOM_SCALE_FACTOR,
            thickness,
            length,
            color,
        ),
    }
}

    fn draw_hovered_room(&self) {
        let world_mouse = self.mouse_world_pos();
        for room in &self.world.rooms {
            let rect = scaled_room_rect(room);
            if rect.contains(world_mouse) {
                let inset = ROOM_LINE_INSET * ROOM_SCALE_FACTOR;

                // Choose highlight color based on mode
                let color = match self.mode {
                    WorldEditorMode::DeletingRoom => HIGHLIGHT_ERROR_COLOR,
                    _ => HIGHLIGHT_COLOR,
                };

                draw_rectangle(
                    rect.x + inset / 2.0,
                    rect.y + inset / 2.0,
                    rect.w - inset,
                    rect.h - inset,
                    color,
                );

                break; // only highlight one room
            }
        }
    }

    fn draw_room_names(&self) {
        set_default_camera(); // draw in screen space

        for room in &self.world.rooms {
            let rect = scaled_room_rect(room);

            // Convert room center to screen coordinates
            let screen_pos = self.camera.world_to_screen(rect.point() + vec2(rect.w / 2.0, rect.h / 2.0));

            // Base font size in pixels
            let base_font_size: f32 = 50.0;

            // Scale based on room size (average of width and height)
            let room_scale = (rect.w + rect.h) / 2.0 / 60.0; 

            let font_size = base_font_size * room_scale * (self.camera.zoom.x * 100.0); 

            // Determine rotation: rotate 90° if room is taller than wide
            let rotation = if rect.h > rect.w { std::f32::consts::FRAC_PI_2 } else { 0.0 };

            let text_dim = measure_text(&room.name, None, font_size as u16, 1.0);

            // Compute proper offset to center text visually
            let offset = if rotation != 0.0 {
                // rotated 90°: width/height swap
                vec2(text_dim.height / 2.0, -text_dim.width / 2.0)
            } else {
                vec2(text_dim.width / 2.0, text_dim.height / 2.0)
            };

            draw_text_ex(
                &room.name,
                screen_pos.x - offset.x,
                screen_pos.y + offset.y,
                TextParams {
                    font_size: font_size as u16,
                    color: BLACK,
                    rotation,
                    ..Default::default()
                },
            );
        }

        set_camera(&self.camera); // back to world camera
    }
    
    fn draw_grid(&self) {
        let step = ROOM_SCALE_FACTOR;
        let line_thickness = LINE_THICKNESS_MULTIPLIER / 2.0 / self.camera.zoom.x;

        let cam_pos = self.camera.target;
        let screen_w = screen_width() / self.camera.zoom.x;
        let screen_h = screen_height() / self.camera.zoom.y;

        let start_x = ((cam_pos.x - screen_w / 2.0) / step).floor() * step;
        let start_y = ((cam_pos.y - screen_h / 2.0) / step).floor() * step;
        let end_x = cam_pos.x + screen_w / 2.0 + step;
        let end_y = cam_pos.y + screen_h / 2.0 + step;

        let mut x = start_x;
        while x <= end_x {
            draw_line(x, start_y, x, end_y, line_thickness, GRID_LINE_COLOR);
            x += step;
        }

        let mut y = start_y;
        while y <= end_y {
            draw_line(start_x, y, end_x, y, line_thickness, GRID_LINE_COLOR);
            y += step;
        }
    }

    fn draw_placing_preview(&self) {
        if let (Some(start), Some(end)) = (self.placing_start, self.placing_end) {
            let (top_left, size) = rect_from_points(start, end);
            let color = if self.intersects_existing_room(top_left, size) { HIGHLIGHT_ERROR_COLOR } else { HIGHLIGHT_COLOR };
            let inset = ROOM_LINE_INSET * ROOM_SCALE_FACTOR;
            draw_rectangle_lines(
                top_left.x * ROOM_SCALE_FACTOR + inset / 2.0,
                top_left.y * ROOM_SCALE_FACTOR + inset / 2.0,
                size.x * ROOM_SCALE_FACTOR - inset,
                size.y * ROOM_SCALE_FACTOR - inset,
                HOVER_LINE_THICKNESS / self.camera.zoom.x,
                color,
            );
        } else {
            let hover_tile = self.snap_to_grid(self.mouse_world_pos() / ROOM_SCALE_FACTOR);
            let color = if self.intersects_existing_room(hover_tile, vec2(1.0, 1.0)) {
                HIGHLIGHT_ERROR_COLOR
            } else {
                HIGHLIGHT_COLOR
            };
            draw_rectangle(
                hover_tile.x * ROOM_SCALE_FACTOR,
                hover_tile.y * ROOM_SCALE_FACTOR,
                ROOM_SCALE_FACTOR,
                ROOM_SCALE_FACTOR,
                color,
            );
        }
    }

    pub fn update_camera(&mut self, dt: f32) {
        let mut direction = vec2(0.0, 0.0);

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { direction.y -= 1.0; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { direction.y += 1.0; }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { direction.x -= 1.0; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { direction.x += 1.0; }

        if direction.length_squared() > 0.0 {
            self.camera.target += direction.normalize() * PAN_SPEED * dt;
        }

        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let zoom_speed = ZOOM_SPEED_FACTOR * self.camera.zoom.x;
            let new_zoom = (self.camera.zoom.x + scroll * zoom_speed).clamp(MIN_ZOOM, MAX_ZOOM);
            self.show_grid = new_zoom >= MAX_ZOOM / 2.0;
            self.camera.zoom = vec2(new_zoom, new_zoom);
        }
    }

    pub fn center_on_room(&mut self, room_idx: usize) {
        let room = &self.world.rooms[room_idx];
        self.camera = Self::compute_camera_for_room(room);
    }

    fn compute_camera_for_room(room: &Room) -> Camera2D {
        let room_size = room.size();
        let room_scaled_size = room_size * ROOM_SCALE_FACTOR;

        let max_dim = room_scaled_size.x.max(room_scaled_size.y);
        let zoom = WORLD_EDITOR_ZOOM_FACTOR / max_dim;

        Camera2D {
            target: (room.position + room_size / 2.0) * ROOM_SCALE_FACTOR,
            zoom: vec2(zoom, zoom),
            ..Default::default()
        }
    }
}

/// Helper: returns rect scaled for drawing
fn scaled_room_rect(room: &Room) -> Rect {
    let size = room.size();
    Rect::new(
        room.position.x * ROOM_SCALE_FACTOR,
        room.position.y * ROOM_SCALE_FACTOR,
        size.x * ROOM_SCALE_FACTOR,
        size.y * ROOM_SCALE_FACTOR,
    )
}

/// Helper: returns unscaled rect for intersection check
// fn room_rect(room: &Room) -> Rect {
//     let size = room.size();
//     Rect::new(room.position.x, room.position.y, size.x, size.y)
// }

/// Helper: compute top-left and size from any two points
fn rect_from_points(p1: Vec2, p2: Vec2) -> (Vec2, Vec2) {
    let top_left = vec2(p1.x.min(p2.x), p1.y.min(p2.y));
    let size = vec2((p1.x - p2.x).abs() + 1.0, (p1.y - p2.y).abs() + 1.0);
    (top_left, size)
}