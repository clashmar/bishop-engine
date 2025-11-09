// editor/src/world/world_editor.rs
use crate::miniquad::CursorIcon;
use macroquad::miniquad::window::set_mouse_cursor;
use crate::gui::menu_panel::*;
use crate::gui::mode_selector::*;
use crate::controls::controls::Controls;
use crate::editor_assets::editor_assets::*;
use crate::{editor_camera_controller::{EditorCameraController}, canvas::grid};
use crate::world::coord;
use once_cell::sync::Lazy;
use engine_core::game::game::Game;
use engine_core::world::world::*;
use engine_core::global::{self, *};
use engine_core::world::room::*;
use engine_core::ui::widgets::*;
use macroquad::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const LINE_THICKNESS_MULTIPLIER: f32 = 0.01;
const HIGHLIGHT_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.5);
const HIGHLIGHT_ERROR_COLOR: Color = Color::new(1.0, 0.0, 0.0, 0.5);
const ROOM_LINE_INSET: f32 = 0.5;
const HOVER_LINE_THICKNESS: f32 = 0.02;

#[derive(Clone, Copy, PartialEq, EnumIter)]
pub enum WorldEditorMode {
    SelectRoom,
    CreateRoom,
    DeleteRoom,
}

impl ModeInfo for WorldEditorMode {
    fn label(&self) -> &'static str {
        match self {
            WorldEditorMode::SelectRoom => "Select: S",
            WorldEditorMode::CreateRoom => "Create Room: C",
            WorldEditorMode::DeleteRoom => "Delete Room: D",
        }
    }
    fn icon(&self) -> &'static Texture2D {
        match self {
            WorldEditorMode::SelectRoom => &SELECT_ICON,
            WorldEditorMode::CreateRoom => &CREATE_ICON,
            WorldEditorMode::DeleteRoom => &DELETE_ICON,
        }
    }
    fn shortcut(self) -> Option<fn() -> bool> {
        match self {
            WorldEditorMode::SelectRoom => Some(Controls::s),
            WorldEditorMode::CreateRoom => Some(Controls::c),
            WorldEditorMode::DeleteRoom => Some(Controls::d),
        }
    }
}

pub struct WorldEditor {
    mode: WorldEditorMode,
    mode_selector: ModeSelector<WorldEditorMode>,
    active_rects: Vec<Rect>,
    show_grid: bool,
    placing_start: Option<Vec2>,
    placing_end: Option<Vec2>, 
    tile_size_id: WidgetId,
}

impl WorldEditor {
    pub fn new() -> Self {
        let active_rects: Vec<Rect> = Vec::new();
        let mode = WorldEditorMode::SelectRoom;

        Self { 
            mode,
            mode_selector: ModeSelector {
                current: mode,
                options: *ALL_MODES,
            },
            active_rects,
            show_grid: true,
            placing_start: None,
            placing_end: None,
            tile_size_id: WidgetId::default(),
        }
    }

    /// Returns `Some(room_id)` if a room is clicked on.
    pub async fn update(&mut self, camera: &mut Camera2D, world: &mut World) -> Option<RoomId> {
        world.link_all_exits();

        self.handle_mouse_cursor();
        self.handle_shortcuts();

        match self.mode {
            WorldEditorMode::SelectRoom => self.update_selecting_mode(camera, world),
            WorldEditorMode::CreateRoom => self.update_placing_mode(camera, world),
            WorldEditorMode::DeleteRoom => self.update_deleting_mode(camera, world),
        }
    }

    fn update_selecting_mode(&mut self, camera: &Camera2D, world: &mut World) -> Option<RoomId> {
        if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
            let world_mouse = coord::mouse_world_pos(camera);
            for room in &world.rooms {
                let rect = scaled_room_rect(room);
                if rect.contains(world_mouse) {
                    return Some(room.id);
                }
            }
        }
        None
    }

    fn update_deleting_mode(&mut self, camera: &Camera2D, world: &mut World) -> Option<RoomId> {
        if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
            let world_mouse = coord::mouse_world_pos(camera);
            for room in &world.rooms {
                let rect = scaled_room_rect(room);
                if rect.contains(world_mouse) {
                    self.delete_room(world, room.id);
                    return None;
                }
            }
        }
        None
    }

    fn update_placing_mode(&mut self, camera: &Camera2D, world: &mut World) -> Option<RoomId> {
        if self.is_mouse_over_ui() {
            return None;
        }
        
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
                if !self.intersects_existing_room(&world.rooms, top_left, size) {
                    // Create the room and get its id back.
                    let new_id = self.place_room_from_drag(world, top_left, size);
                    self.reset_placing();
                    self.mode = WorldEditorMode::SelectRoom;
                    return Some(new_id);
                }
                // Overlap – just abort placement.
                self.reset_placing();
            }
        }
        None
    }

    fn intersects_existing_room(
        &self,
        rooms: &Vec<Room>,
        top_left: Vec2,
        size: Vec2,
    ) -> bool {
        let bounds: Vec<(Vec2, Vec2)> = rooms
            .iter()
            .map(|rm| (rm.position, rm.size))
            .collect();

        coord::overlaps_existing_rooms(top_left * tile_size(), size * tile_size(), &bounds)
    }

    fn reset_placing(&mut self) {
        self.placing_start = None;
        self.placing_end = None;
    }

    pub fn draw(
        &mut self, 
        world_id: WorldId,
        camera: &Camera2D, 
        game: &mut Game
    ) {
        set_camera(camera);
        clear_background(LIGHTGRAY);

        let world = game.get_world(world_id);
        let rooms = &world.rooms;

        grid::draw_grid(camera);

        self.draw_rooms(camera, rooms);
        self.draw_exits(rooms);

        match self.mode {
            WorldEditorMode::SelectRoom => {
                if !self.is_mouse_over_ui() {
                    self.draw_hovered_room(camera, rooms);
                }
            }
            WorldEditorMode::DeleteRoom => {
                if !self.is_mouse_over_ui() {
                    self.draw_hovered_room(camera, rooms);
                }
            }
            WorldEditorMode::CreateRoom => {
                if !self.is_mouse_over_ui() {
                    self.draw_placing_preview(camera, rooms);
                }
            }
        }

        self.draw_room_names(camera, rooms); 
        self.draw_ui(camera, game);
        
        // Static UI camera
        set_default_camera();
        self.draw_coordinates(camera);
    }

    pub fn draw_rooms(&self, camera: &Camera2D, rooms: &Vec<Room>) {
        for room in rooms {
            let rect = scaled_room_rect(room);
            let inset = ROOM_LINE_INSET * tile_size();

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

    fn draw_exits(&self, rooms: &Vec<Room>) {
        for room in rooms {
            for exit in &room.exits {
                let exit_world_coord = (room.position / tile_size()) + exit.position;
                // Decide color based on whether it's linked
                let color = if exit.target_room_id.is_some() {
                    GREEN
                } else {
                    RED
                };
                self.draw_exit_marker(exit_world_coord, exit.direction, color);
            }
        }
    }

    fn draw_exit_marker(&self, exit_world_coord: Vec2, dir: ExitDirection, color: Color) {
        let thickness = 4.0;
        let length = tile_size();
        let offset = 1.0; 

        match dir {
            ExitDirection::Up => draw_rectangle(
                exit_world_coord.x * tile_size(),
                exit_world_coord.y * tile_size() + tile_size(),
                length,
                thickness,
                color,
            ),
            ExitDirection::Down => draw_rectangle(
                exit_world_coord.x * tile_size(),
                exit_world_coord.y * tile_size() - thickness + offset,
                length,
                thickness,
                color,
            ),
            ExitDirection::Left => draw_rectangle(
                (exit_world_coord.x + 1.0) * tile_size() - offset,
                exit_world_coord.y * tile_size(),
                thickness,
                length,
                color,
            ),
            ExitDirection::Right => draw_rectangle(
                (exit_world_coord.x - 1.0) * tile_size() + tile_size() - thickness + offset,
                exit_world_coord.y * tile_size(),
                thickness,
                length,
                color,
            ),
        }
    }

    fn draw_hovered_room(&self, camera: &Camera2D, rooms: &Vec<Room>) {
        let world_mouse = coord::mouse_world_pos(camera);
        for room in rooms {
            let rect = scaled_room_rect(room);
            if rect.contains(world_mouse) {
                let inset = ROOM_LINE_INSET * tile_size();

                // Choose highlight color based on mode
                let color = match self.mode {
                    WorldEditorMode::DeleteRoom => HIGHLIGHT_ERROR_COLOR,
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

    fn draw_room_names(&self, camera: &Camera2D, rooms: &Vec<Room>) {
        set_default_camera(); // draw in screen space

        for room in rooms {
            let rect = scaled_room_rect(room);

            // Screen coordinates of room center
            let screen_pos = camera.world_to_screen(rect.point() + rect.size() / 2.0);

            let text_len = room.name.len() as f32;

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
                &room.name,
                screen_pos.x - offset.x,
                screen_pos.y + offset.y,
                TextParams {
                    font_size: font_size as u16,
                    color: BLACK,
                    rotation,
                    ..Default::default()
                });
            }
        set_camera(camera); // back to world camera
    }

    fn draw_placing_preview(&self, camera: &Camera2D, rooms: &Vec<Room>) {
        if let (Some(start), Some(end)) = (self.placing_start, self.placing_end) {
            let (top_left, size) = rect_from_points(start, end);
            let color = if self.intersects_existing_room(rooms, top_left, size) { HIGHLIGHT_ERROR_COLOR } else { HIGHLIGHT_COLOR };
            let inset = ROOM_LINE_INSET * tile_size();
            draw_rectangle_lines(
                top_left.x * tile_size() + inset / 2.0,
                top_left.y * tile_size() + inset / 2.0,
                size.x * tile_size() - inset,
                size.y * tile_size() - inset,
                HOVER_LINE_THICKNESS / camera.zoom.x,
                color,
            );
        } else {
            let hover_tile = coord::snap_to_grid(coord::mouse_world_grid(camera));
            let color = if self.intersects_existing_room(rooms, hover_tile, vec2(1.0, 1.0)) {
                HIGHLIGHT_ERROR_COLOR
            } else {
                HIGHLIGHT_COLOR
            };
            draw_rectangle(
                hover_tile.x * tile_size(),
                hover_tile.y * tile_size(),
                tile_size(),
                tile_size(),
                color,
            );
        }
    }

    fn draw_ui(&mut self, camera: &Camera2D, game: &mut Game) {
        self.active_rects.clear();

        // Static camera
        set_default_camera();

        // Top menu panel
        self.register_rect(draw_top_panel_full());

        // Mode selector
        if self.mode_selector.draw().1 {
            self.mode = self.mode_selector.current;
        }

        // Tile size field
        let tile_size_rect = Rect::new(
            screen_width() - 50.0,
            10.0,                  
            40.0,                 
            30.0,                 
        );
        
        let new_size = gui_input_number_f32(self.tile_size_id, tile_size_rect, game.tile_size);
        if new_size != game.tile_size {
            let old_size = game.tile_size;
            global::update_tile_size(game, old_size, new_size);
        }

        set_camera(camera); // Back to world camera
    }

    pub fn center_on_room(&mut self, camera: &mut Camera2D, room: &Room) {
        *camera = EditorCameraController::camera_for_room(room.size, room.position);
    }

    fn handle_shortcuts(&mut self) {
        if Controls::g() {
            self.show_grid = !self.show_grid;
        }

        for mode in WorldEditorMode::iter() {
            if let Some(is_pressed) = mode.shortcut() {
                if is_pressed() && !input_is_focused() {
                    self.mode = mode;
                    self.mode_selector.current = mode;
                    break;
                }
            }
        }
    }

    #[inline]
    fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }

    fn is_mouse_over_ui(&self) -> bool {
        let mouse_screen: Vec2 = mouse_position().into();
        self.active_rects.iter().any(|r| r.contains(mouse_screen))
    }

    fn handle_mouse_cursor(&self) {
        if self.is_mouse_over_ui() {
            set_mouse_cursor(CursorIcon::Default);
        } else {
            match self.mode {
                WorldEditorMode::SelectRoom => {
                    set_mouse_cursor(CursorIcon::Pointer);
                }
                WorldEditorMode::CreateRoom => {
                    set_mouse_cursor(CursorIcon::Crosshair);
                }
                WorldEditorMode::DeleteRoom => {
                    set_mouse_cursor(CursorIcon::Crosshair);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.mode = WorldEditorMode::SelectRoom;
        self.placing_start = None;
        self.placing_end = None;
        self.active_rects.clear();
        self.show_grid = true;
    }
}

/// Returns rect scaled for drawing
fn scaled_room_rect(room: &Room) -> Rect {
    let size = room.size;
    Rect::new(
        room.position.x,
        room.position.y,
        size.x * tile_size(),
        size.y * tile_size(),
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

/// A slice of all the modes.
static ALL_MODES: Lazy<&'static [WorldEditorMode]> = Lazy::new(|| {
    Box::leak(Box::new(
        WorldEditorMode::iter().collect::<Vec<_>>()
    ))
});