use core::{constants::TILE_SIZE, world::{room::{ExitDirection, RoomMetadata}, world::World}};
use macroquad::prelude::*;
use uuid::Uuid;
use crate::camera_controller::{self, CameraController};
use crate::{gui::{ui_element::WorldUiElement, world_ui::WorldNameUi}};
use crate::world::coord;

const HIGHLIGHT_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.5);
const HIGHLIGHT_ERROR_COLOR: Color = Color::new(1.0, 0.0, 0.0, 0.5);
const LINE_THICKNESS_MULTIPLIER: f32 = 0.02;
const ROOM_LINE_INSET: f32 = 0.5;
const GRID_LINE_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.2);
const HOVER_LINE_THICKNESS: f32 = 0.02;

pub enum WorldEditorMode {
    Selecting,
    PlacingRoom,
    DeletingRoom,
}

pub struct WorldEditor {
    mode: WorldEditorMode,
    ui_elements: Vec<Box<dyn WorldUiElement>>,
    show_grid: bool,
    placing_start: Option<Vec2>,
    placing_end: Option<Vec2>, 
}

impl WorldEditor {
    pub fn new() -> Self {
        let mut ui_elements: Vec<Box<dyn WorldUiElement>> = Vec::new();
        ui_elements.push(Box::new(WorldNameUi::new()));

        Self { 
            mode: WorldEditorMode::Selecting,
            ui_elements,
            show_grid: true,
            placing_start: None,
            placing_end: None,
        }
    }

    /// Returns `Some(room_id)` if a room is clicked on.
    pub async fn update(&mut self, camera: &mut Camera2D, world: &mut World) -> Option<Uuid> {
        world.link_all_exits();
        self.handle_ui_clicks(world).await;

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
            WorldEditorMode::Selecting => self.update_selecting_mode(camera, world),
            WorldEditorMode::PlacingRoom => self.update_placing_mode(camera, world),
            WorldEditorMode::DeletingRoom => self.update_deleting_mode(camera, world),
        }
    }

    async fn handle_ui_clicks(&mut self, world: &mut World) {
        if is_mouse_button_pressed(MouseButton::Left) {
            for element in &self.ui_elements {
                if let Some(rect) = element.rect(world) { // pass `world`
                    if mouse_over_rect(rect) {
                        element.on_click(world).await;
                        break; // only handle one click
                    }
                }
            }
        }
    }

    fn update_selecting_mode(&mut self, camera: &Camera2D, world: &mut World) -> Option<Uuid> {
        if is_mouse_button_pressed(MouseButton::Left) {
            let world_mouse = coord::mouse_world_pos(camera);
            for meta in &world.rooms_metadata {
                let rect = scaled_room_rect(meta);
                if rect.contains(world_mouse) {
                    return Some(meta.id);
                }
            }
        }
        None
    }

    fn update_deleting_mode(&mut self, camera: &Camera2D, world: &mut World) -> Option<Uuid> {
        if is_mouse_button_pressed(MouseButton::Left) {
            let world_mouse = coord::mouse_world_pos(camera);
            for meta in &world.rooms_metadata {
                let rect = scaled_room_rect(meta);
                if rect.contains(world_mouse) {
                    self.delete_room(world, meta.id);
                    return None;
                }
            }
        }
        None
    }

    fn update_placing_mode(&mut self, camera: &Camera2D, world: &mut World) -> Option<Uuid> {
        let mouse_tile = coord::snap_to_grid(coord::mouse_world_grid(camera));

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
                if !self.intersects_existing_room(&world.rooms_metadata, top_left, size) {
                    // Create the room and get its UUID back.
                    let new_id = self.place_room_from_drag(world, top_left, size);
                    self.reset_placing();
                    self.mode = WorldEditorMode::Selecting;
                    return Some(new_id);
                }
                // Overlap – just abort placement.
                self.reset_placing();
            }
        }
        None
    }

    fn intersects_existing_room(&self, rooms_metadata: &Vec<RoomMetadata>, top_left: Vec2, size: Vec2) -> bool {
        let a_left = top_left.x;
        let a_right = top_left.x + size.x;
        let a_top = top_left.y;
        let a_bottom = top_left.y + size.y;

        for room_metadata in rooms_metadata {
            let b_left = room_metadata.position.x;
            let b_right = room_metadata.position.x + room_metadata.size.x;
            let b_top = room_metadata.position.y;
            let b_bottom = room_metadata.position.y + room_metadata.size.y;

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

    pub fn draw(&mut self, camera: &Camera2D, world: &World) {
        set_camera(camera);
        clear_background(LIGHTGRAY);

        let rooms_metadata = &world.rooms_metadata;

        self.draw_grid(camera);

        self.draw_rooms(camera, rooms_metadata);
        self.draw_unlinked_exits(rooms_metadata);

        // Highlight hovered room in select or delete mode
        match self.mode {
            WorldEditorMode::Selecting | 
            WorldEditorMode::DeletingRoom => self.draw_hovered_room(camera, rooms_metadata),
            _ => {},
        }

        if let WorldEditorMode::PlacingRoom = self.mode {
            self.draw_placing_preview(camera, rooms_metadata);
        }

        self.draw_room_names(camera, rooms_metadata); 
        self.draw_ui(camera, world);
        
        set_default_camera();
        self.draw_coordinates(camera);
    }

    pub fn draw_rooms(&self, camera: &Camera2D, rooms_metadata: &Vec<RoomMetadata>) {
        for room_metadata in rooms_metadata {
            let rect = scaled_room_rect(room_metadata);
            let inset = ROOM_LINE_INSET * TILE_SIZE;

            // Draw the room outline
            draw_rectangle_lines(
                rect.x + inset / 2.0,
                rect.y + inset / 2.0,
                rect.w - inset,
                rect.h - inset,
                LINE_THICKNESS_MULTIPLIER / camera.zoom.x,
                BLUE,
            );
        }
    }

    fn draw_unlinked_exits(&self, rooms_metadata: &Vec<RoomMetadata>) {
        for room_metadata in rooms_metadata {
            for (exit_world_pos, dir) in room_metadata.world_exit_positions() {
                for exit in &room_metadata.exits {
                    let pos = room_metadata.position + exit.position;

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
        let thickness = 4.0;
        let length = TILE_SIZE;
        let offset = 1.0; 

        match dir {
            ExitDirection::Up => draw_rectangle(
                exit_world_pos.x * TILE_SIZE,
                exit_world_pos.y * TILE_SIZE + TILE_SIZE,
                length,
                thickness,
                color,
            ),
            ExitDirection::Down => draw_rectangle(
                exit_world_pos.x * TILE_SIZE,
                exit_world_pos.y * TILE_SIZE - thickness + offset,
                length,
                thickness,
                color,
            ),
            ExitDirection::Left => draw_rectangle(
                (exit_world_pos.x + 1.0) * TILE_SIZE - offset,
                exit_world_pos.y * TILE_SIZE,
                thickness,
                length,
                color,
            ),
            ExitDirection::Right => draw_rectangle(
                (exit_world_pos.x - 1.0) * TILE_SIZE + TILE_SIZE - thickness + offset,
                exit_world_pos.y * TILE_SIZE,
                thickness,
                length,
                color,
            ),
        }
    }

    fn draw_hovered_room(&self, camera: &Camera2D, rooms_metadata: &Vec<RoomMetadata>) {
        let world_mouse = coord::mouse_world_pos(camera);
        for room_metadata in rooms_metadata {
            let rect = scaled_room_rect(room_metadata);
            if rect.contains(world_mouse) {
                let inset = ROOM_LINE_INSET * TILE_SIZE;

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

    fn draw_room_names(&self, camera: &Camera2D, rooms_metadata: &Vec<RoomMetadata>) {
        set_default_camera(); // draw in screen space

        for room_metadata in rooms_metadata {
            let rect = scaled_room_rect(room_metadata);

            // Screen coordinates of room center
            let screen_pos = camera.world_to_screen(rect.point() + rect.size() / 2.0);

            let text_len = room_metadata.name.len() as f32;

            // Base text size
            let base_font_size: f32 = 40.0;

            // Scale based on room size and camera zoom
            let room_scale = (rect.w + rect.h) / 2.0 / 60.0;
            let zoom_factor = camera.zoom.x * 100.0;
            let font_size = (base_font_size * room_scale * zoom_factor).clamp(10.0, 200.0);

            // Rotation: vertical if tall
            let rotation = if rect.h > rect.w { std::f32::consts::FRAC_PI_2 } else { 0.0 };

            // Approximate text half-size
            let half_width = font_size * text_len * 0.25; 
            let half_height = font_size * 1.5;           

            // Offset along rotated axes
            let offset = if rotation != 0.0 {
                vec2(half_height * 0.1, -half_width * 0.85) 
            } else {
                vec2(half_width * 0.875, half_height * 0.1)
            };

            // Draw
            draw_text_ex(
                &room_metadata.name,
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
    set_camera(camera); // back to world camera
}
    
    fn draw_grid(&mut self, camera: &Camera2D) {
        let scalar = CameraController::scalar_zoom(camera);
        self.show_grid = scalar >= camera_controller::MIN_ZOOM * 2.0;
        if !self.show_grid {
            return;
        }

        let step = TILE_SIZE;
        let line_thickness = LINE_THICKNESS_MULTIPLIER / 2.0 / camera.zoom.x;

        let cam_pos = camera.target;
        let screen_w = screen_width() / camera.zoom.x;
        let screen_h = screen_height() / camera.zoom.y;

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

    fn draw_placing_preview(&self, camera: &Camera2D, rooms_metadata: &Vec<RoomMetadata>) {
        if let (Some(start), Some(end)) = (self.placing_start, self.placing_end) {
            let (top_left, size) = rect_from_points(start, end);
            let color = if self.intersects_existing_room(rooms_metadata, top_left, size) { HIGHLIGHT_ERROR_COLOR } else { HIGHLIGHT_COLOR };
            let inset = ROOM_LINE_INSET * TILE_SIZE;
            draw_rectangle_lines(
                top_left.x * TILE_SIZE + inset / 2.0,
                top_left.y * TILE_SIZE + inset / 2.0,
                size.x * TILE_SIZE - inset,
                size.y * TILE_SIZE - inset,
                HOVER_LINE_THICKNESS / camera.zoom.x,
                color,
            );
        } else {
            let hover_tile = coord::snap_to_grid(coord::mouse_world_grid(camera));
            let color = if self.intersects_existing_room(rooms_metadata, hover_tile, vec2(1.0, 1.0)) {
                HIGHLIGHT_ERROR_COLOR
            } else {
                HIGHLIGHT_COLOR
            };
            draw_rectangle(
                hover_tile.x * TILE_SIZE,
                hover_tile.y * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                color,
            );
        }
    }

    fn draw_ui(&self, camera: &Camera2D, world: &World) {
        set_default_camera(); // screen space

        for element in &self.ui_elements {
            element.draw(world);
        }

        set_camera(camera); // back to world camera
    }

    pub fn center_on_room(&mut self, camera: &mut Camera2D, room_metadata: &RoomMetadata) {
        *camera = CameraController::camera_for_room(room_metadata.size, room_metadata.position);
    }
}

pub fn mouse_over_rect(rect: Rect) -> bool {
    let mouse_pos = mouse_position();
    rect.contains(vec2(mouse_pos.0, mouse_pos.1))
}

/// Returns rect scaled for drawing
fn scaled_room_rect(room_metadata: &RoomMetadata) -> Rect {
    let size = room_metadata.size;
    Rect::new(
        room_metadata.position.x * TILE_SIZE,
        room_metadata.position.y * TILE_SIZE,
        size.x * TILE_SIZE,
        size.y * TILE_SIZE,
    )
}

/// Compute top-left and size from any two points
fn rect_from_points(p1: Vec2, p2: Vec2) -> (Vec2, Vec2) {
    let top_left = vec2(p1.x.min(p2.x), p1.y.min(p2.y));
    let size = vec2(
        (p1.x - p2.x).abs().floor() + 1.0,
        (p1.y - p2.y).abs().floor() + 1.0,
    );

    (top_left, size)
}