// editor/src/room/drawing.rs
use crate::editor_camera_controller::*;
use crate::ecs::transform::{Pivot, Transform};
use crate::gui::gui_constants::*;
use crate::room::room_editor::*;
use crate::gui::menu_bar::*;
use crate::world::coord;
use engine_core::animation::animation_system::CurrentFrame;
use engine_core::animation::animation_clip::Animation;
use engine_core::assets::asset_manager::AssetManager;
use engine_core::camera::game_camera::RoomCamera;
use engine_core::rendering::render_room::*;
use engine_core::lighting::light::Light;
use engine_core::assets::sprite::Sprite;
use engine_core::game::game::GameCtxMut;
use engine_core::lighting::glow::Glow;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::component::*;
use engine_core::ui::widgets::*;
use engine_core::world::room::*;
use engine_core::ecs::ecs::Ecs;
use engine_core::ui::text::*;
use macroquad::prelude::*;

const PLACEHOLDER_OPACITY: f32 = 0.2;
fn thickness(grid_size: f32) -> f32 { (grid_size * 0.175).max(1.0) }

impl RoomEditor {
    /// Draw static UI for the scene editor
    pub fn draw_ui(
        &mut self, 
        game_ctx: &mut GameCtxMut,
        camera: &Camera2D,
    ) {
        // Reset to static camera
        set_default_camera();

        self.draw_coordinates(camera, game_ctx.cur_world.current_room().unwrap(), game_ctx.cur_world.grid_size);

        match self.mode {
            RoomEditorMode::Tilemap => {
                // Mode selector
                if self.mode_selector.draw().1 {
                    self.mode = self.mode_selector.current;
                }
            }
            RoomEditorMode::Scene => {
                // Top menu background
                self.register_rect(draw_top_panel_full());
                
                // Draw inspector
                self.create_entity_requested = self.inspector.draw(
                    game_ctx
                );

                // Mode selector (menu bar)
                let (mode_rect, changed) = self.mode_selector.draw();
                if changed {
                    self.mode = self.mode_selector.current;
                }

                // Play‑test button (menu bar)
                let play_label = "Play";
                let play_width = measure_text_ui(play_label, HEADER_FONT_SIZE_20, 1.0).width + WIDGET_PADDING * 2.0;
                let play_x = mode_rect.x + mode_rect.w + WIDGET_SPACING;
                let play_rect = Rect::new(play_x, INSET, play_width, BTN_HEIGHT);

                if menu_button(play_rect, play_label, false) {
                    self.request_play = true;
                }
            }
        }
    }

    /// Draw the cursor coordinates in world space.
    pub fn draw_coordinates(&self, camera: &Camera2D, _room: &Room, grid_size: f32) {
        let world_grid = coord::mouse_world_grid(camera, grid_size);
        
        let txt = format!(
            "({:.0}, {:.0})",
            world_grid.x, world_grid.y,
        );

        let txt_metrics = measure_text_ui(&txt, DEFAULT_FONT_SIZE_16, 1.0);
        let margin = 10.0;

        let x = (screen_width() - txt_metrics.width) / 2.0;
        let y = screen_height() - margin;

        draw_text_ui(&txt, x, y, DEFAULT_FONT_SIZE_16, BLUE);
    }

    /// Draw viewport rectangles for all cameras in the room when a camera is selected.
    /// The selected camera is drawn in yellow, others in pink.
    pub fn draw_camera_viewport(
        &self,
        editor_cam: &Camera2D,
        ecs: &Ecs,
        selected: Entity,
        room_id: RoomId,
    ) {
        // Only draw viewports if the selected entity is a camera
        if !ecs.has::<RoomCamera>(selected) {
            return;
        }

        let cam_store = ecs.get_store::<RoomCamera>();
        let pos_store = ecs.get_store::<Transform>();
        let room_store = ecs.get_store::<CurrentRoom>();

        let editor_scalar = EditorCameraController::scalar_zoom(editor_cam);
        const BASE_THICKNESS: f32 = 1.;
        const THICKNESS_SCALE: f32 = 0.01;
        let thickness = BASE_THICKNESS * (THICKNESS_SCALE / editor_scalar).max(1.0);

        let bl = editor_cam.screen_to_world(vec2(0.0, 0.0));
        let tr = editor_cam.screen_to_world(vec2(screen_width(), screen_height()));
        let editor_w = (tr.x - bl.x).abs();
        let editor_h = (tr.y - bl.y).abs();

        // Collect all cameras in this room
        for (entity, room_cam) in cam_store.data.iter() {
            // Only draw cameras in this room
            if let Some(CurrentRoom(id)) = room_store.get(*entity) {
                if *id != room_id {
                    continue;
                }
            } else {
                continue;
            }

            let pos = match pos_store.get(*entity) {
                Some(p) => p.position,
                None => continue,
            };

            let factor_x = editor_cam.zoom.x / room_cam.zoom.x;
            let factor_y = editor_cam.zoom.y / room_cam.zoom.y;

            let viewport_w = editor_w * factor_x;
            let viewport_h = editor_h * factor_y;

            let half = vec2(viewport_w, viewport_h) * 0.5;
            let top_left = pos - half;

            // Selected camera is yellow, others are dimmer cyan
            let color = if *entity == selected {
                YELLOW
            } else {
                PINK
            };

            draw_rectangle_lines(
                top_left.x,
                top_left.y,
                viewport_w,
                viewport_h,
                thickness,
                color,
            );
        }
    }
}

/// Draw the outline of the collider for an entity if it has one.
pub fn draw_collider(
    ecs: &Ecs,
    entity: Entity,
) {
    if let Some((width, height)) = ecs
        .get_store::<Collider>()
        .get(entity)
        .filter(|c| c.width > 0.0 && c.height > 0.0)
        .map(|c| (c.width, c.height)) {
            let transform = match ecs.get_store::<Transform>().get(entity) {
                Some(t) => t,
                None => return,
            };

            // Apply pivot offset to collider position
            let draw_pos = pivot_adjusted_position(transform.position, vec2(width, height), transform.pivot);
            draw_rectangle_lines(draw_pos.x, draw_pos.y, width, height, 2.0, PINK);
     }
}

/// Returns a `Rect` hitbox for an entity based on its sprite if it has one,
/// otherwise it returns a hitbox based on the default sprite dimensions.
pub fn entity_hitbox(
    entity: Entity,
    position: Vec2,
    camera: &Camera2D,
    ecs: &Ecs,
    asset_manager: &mut AssetManager,
    grid_size: f32,
) -> Rect {
    let (width, height) = entity_dimensions(ecs, asset_manager, entity, grid_size);

    // Only use the center-offset for pure placeholder entities (Camera/Light without sprites)
    let is_pure_placeholder = ecs.has::<RoomCamera>(entity)
        || (ecs.has::<Light>(entity) && !ecs.has_any::<(Sprite, Animation, CurrentFrame)>(entity));

    let corrected_pos = if is_pure_placeholder {
        position - vec2(grid_size * 0.5, grid_size * 0.5)
    } else {
        // Apply pivot offset for regular entities
        let pivot = ecs
            .get_store::<Transform>()
            .get(entity)
            .map(|t| t.pivot)
            .unwrap_or(Pivot::TopLeft);
        pivot_adjusted_position(position, vec2(width, height), pivot)
    };

    // Convert the two opposite corners of the entity to screen coords
    let top_left = coord::world_to_screen(camera, corrected_pos);
    let bottom_right = coord::world_to_screen(camera, corrected_pos + vec2(width, height));

    // Build the rectangle from those screen‑space points
    let rect_x = top_left.x;
    let rect_y = top_left.y;
    let rect_w = (bottom_right.x - top_left.x).abs();
    let rect_h = (bottom_right.y - top_left.y).abs();

    Rect::new(rect_x, rect_y, rect_w, rect_h)
}

/// Draw an icon for a `RoomCamera`.
pub fn draw_camera_placeholders(ecs: &Ecs, room_id: RoomId, grid_size: f32) {
    let cam_store = ecs.get_store::<RoomCamera>();
    let pos_store = ecs.get_store::<Transform>();
    let room_store = ecs.get_store::<CurrentRoom>();

    let positions: Vec<Vec2> = cam_store
        .data
        .iter()
        .filter_map(|(entity, _room_cam)| {
            let cur_room = room_store.get(*entity)?;
            if cur_room.0 != room_id {
                return None;
            }
            let pos = pos_store.get(*entity)?;
            Some(pos.position)
        })
        .collect();
    
    for pos in positions {
        // Offset the camera placeholder 
        let half_tile = grid_size * 0.5;
        let body = Rect::new(
            pos.x - half_tile,   
            pos.y - half_tile,
            grid_size,
            grid_size,
        );

        let green = Color::new(0.0, 0.89, 0.19, PLACEHOLDER_OPACITY);
        let blue = Color::new(0.0, 0.47, 0.95, PLACEHOLDER_OPACITY);
        let red = Color::new(0.9, 0.16, 0.22, PLACEHOLDER_OPACITY);

        draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness(grid_size), green);

        let finder_w = grid_size * 0.3;
        let finder_h = grid_size * 0.6;
        let finder = Rect::new(
            body.x + thickness(grid_size),                     
            body.y + (body.h - finder_h) / 2.0,
            finder_w,
            finder_h,
        );
        draw_rectangle_lines(finder.x, finder.y, finder.w, finder.h,
                            thickness(grid_size) * 0.75, blue);

        let lens_radius = grid_size * 0.1;
        let lens_center = vec2(
            body.x + body.w - lens_radius * 2.0 - thickness(grid_size),
            body.y + body.h / 2.0,
        );
        draw_circle_lines(lens_center.x, lens_center.y,
                        lens_radius, thickness(grid_size) * 0.75, red);
    }
}

/// Draw an icon for a `Light` that has no other visual component.
pub fn draw_light_placeholders(
    ecs: &Ecs,
    room_id: RoomId,
    grid_size: f32
) {
    let room_store = ecs.get_store::<CurrentRoom>();
    for (entity, _light) in ecs.get_store::<Light>().data.iter() {
        // Only draw placeholders in this room
        if let Some(CurrentRoom(id)) = room_store.get(*entity) {
            if *id != room_id { continue; }
        }

        // Don't draw if there is a Sprite or Animation component
        if ecs.has_any::<(Sprite, Animation)>(*entity) {
            continue;
        }

        if let Some(position) = ecs.get_store::<Transform>().get(*entity) {
            let pos = position.position;

            let half_tile = grid_size * 0.5;
            let body = Rect::new(
                pos.x - half_tile,
                pos.y - half_tile,
                grid_size,
                grid_size,
            );

            let cyan = Color::new(0.0, 0.78, 0.78, PLACEHOLDER_OPACITY);
            let yellow = Color::new(0.94, 0.86, 0.0, PLACEHOLDER_OPACITY);

            // Outer square
            draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness(grid_size), cyan);

            // Lens
            let lens_radius = grid_size * 0.2;
            let lens_center = vec2(
                body.x + body.w / 2.,
                body.y + body.h / 2.,
            );

            draw_circle_lines(
                lens_center.x,
                lens_center.y,
                lens_radius,
                thickness(grid_size) * 0.75,
                yellow,
            );
        }
    }
}

/// Draw a placeholder for a `Glow` that has no other visual component.
pub fn draw_glow_placeholders(
    ecs: &Ecs,
    asset_manager: &mut AssetManager,
    room_id: RoomId,
    grid_size: f32,
) {
    let room_store = ecs.get_store::<CurrentRoom>();
    for (entity, glow) in ecs.get_store::<Glow>().data.iter() {
        // Only draw placeholders in this room
        if let Some(CurrentRoom(id)) = room_store.get(*entity) {
            if *id != room_id { continue; }
        }

        // Don't draw if there is a Sprite or Animation component
        if ecs.has_any::<(Sprite, Animation)>(*entity) {
            continue;
        }

        if let Some(position) = ecs.get_store::<Transform>().get(*entity) {
            let mut pos = position.position;

            if let Some((w, h)) = asset_manager.texture_size(glow.sprite_id) {
                pos = pos + vec2((w / 2.) - grid_size / 2., (h / 2.) - grid_size / 2.);
            }

            let body = Rect::new(
                pos.x,
                pos.y,
                grid_size,
                grid_size,
            );

            let cyan = Color::new(0.0, 0.78, 0.78, PLACEHOLDER_OPACITY);
            let yellow = Color::new(0.94, 0.86, 0.0, PLACEHOLDER_OPACITY);

            // Outer square
            draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness(grid_size), cyan);

            // Lens
            let lens_radius = grid_size * 0.2;
            let lens_center = vec2(
                body.x + body.w / 2.,
                body.y + body.h / 2.,
            );

            draw_circle_lines(
                lens_center.x,
                lens_center.y,
                lens_radius,
                thickness(grid_size) * 0.75,
                yellow,
            );
        }
    }
}

/// Draws a small white dot at the pivot point of the selected entity.
pub fn draw_pivot_marker(ecs: &Ecs, entity: Entity) {
    let transform = match ecs.get_store::<Transform>().get(entity) {
        Some(t) => t,
        None => return,
    };

    const PIVOT_RADIUS: f32 = 1.0;
    draw_circle(transform.position.x, transform.position.y, PIVOT_RADIUS, WHITE);
}

/// Returns true if the entity is a pure placeholder (Camera or Light without visible sprites).
pub fn is_pure_placeholder(ecs: &Ecs, entity: Entity) -> bool {
    ecs.has::<RoomCamera>(entity)
        || (ecs.has::<Light>(entity) && !ecs.has_any::<(Sprite, Animation, CurrentFrame)>(entity))
}

/// Draw exit arrows for all exits in the room.
pub fn draw_exit_placeholders(exits: &[Exit], room_position: Vec2, grid_size: f32) {
    for exit in exits {
        let position = exit.position * grid_size + room_position;
        draw_exit_arrow(position, exit.direction, grid_size);
    }
}

/// Draw all camera viewports in a room.
pub fn draw_all_camera_viewports(
    editor_cam: &Camera2D,
    ecs: &Ecs,
    room_id: RoomId,
) {
    let cam_store = ecs.get_store::<RoomCamera>();
    let pos_store = ecs.get_store::<Transform>();
    let room_store = ecs.get_store::<CurrentRoom>();

    let editor_scalar = EditorCameraController::scalar_zoom(editor_cam);
    const BASE_THICKNESS: f32 = 1.;
    const THICKNESS_SCALE: f32 = 0.01;
    let thickness = BASE_THICKNESS * (THICKNESS_SCALE / editor_scalar).max(1.0);

    let bl = editor_cam.screen_to_world(vec2(0.0, 0.0));
    let tr = editor_cam.screen_to_world(vec2(screen_width(), screen_height()));
    let editor_w = (tr.x - bl.x).abs();
    let editor_h = (tr.y - bl.y).abs();

    for (entity, room_cam) in cam_store.data.iter() {
        if let Some(CurrentRoom(id)) = room_store.get(*entity) {
            if *id != room_id {
                continue;
            }
        } else {
            continue;
        }

        let pos = match pos_store.get(*entity) {
            Some(p) => p.position,
            None => continue,
        };

        let factor_x = editor_cam.zoom.x / room_cam.zoom.x;
        let factor_y = editor_cam.zoom.y / room_cam.zoom.y;

        let viewport_w = editor_w * factor_x;
        let viewport_h = editor_h * factor_y;

        let half = vec2(viewport_w, viewport_h) * 0.5;
        let top_left = pos - half;

        draw_rectangle_lines(
            top_left.x,
            top_left.y,
            viewport_w,
            viewport_h,
            thickness,
            PINK,
        );
    }
}

/// Draw a semi-transparent arrow at the given position indicating exit direction.
pub fn draw_exit_arrow(position: Vec2, direction: ExitDirection, grid_size: f32) {
    draw_exit_arrow_colored(position, direction, grid_size, HIGHLIGHT_GREEN);
}

/// Draw an arrow for an adjacent room's exit (pink color to distinguish from current room).
pub fn draw_adjacent_exit_arrow(position: Vec2, direction: ExitDirection, grid_size: f32) {
    draw_exit_arrow_colored(position, direction, grid_size, YELLOW);
}

/// Draw a selection box rectangle in world space.
pub fn draw_selection_box(start: Vec2, end: Vec2) {
    let min_x = start.x.min(end.x);
    let min_y = start.y.min(end.y);
    let max_x = start.x.max(end.x);
    let max_y = start.y.max(end.y);
    let width = max_x - min_x;
    let height = max_y - min_y;

    // Semi-transparent fill
    draw_rectangle(min_x, min_y, width, height, Color::new(1.0, 1.0, 0.0, 0.1));
    // Yellow outline
    draw_rectangle_lines(min_x, min_y, width, height, 1.0, YELLOW);
}

/// Returns a world-space Rect for an entity based on its sprite or placeholder size.
pub fn entity_world_rect(
    entity: Entity,
    position: Vec2,
    ecs: &Ecs,
    asset_manager: &mut AssetManager,
    grid_size: f32,
) -> Rect {
    let (width, height) = entity_dimensions(ecs, asset_manager, entity, grid_size);

    let is_placeholder = ecs.has::<RoomCamera>(entity)
        || (ecs.has::<Light>(entity) && !ecs.has_any::<(Sprite, Animation, CurrentFrame)>(entity));

    let corrected_pos = if is_placeholder {
        position - vec2(grid_size * 0.5, grid_size * 0.5)
    } else {
        let pivot = ecs
            .get_store::<Transform>()
            .get(entity)
            .map(|t| t.pivot)
            .unwrap_or(Pivot::TopLeft);
        pivot_adjusted_position(position, vec2(width, height), pivot)
    };

    Rect::new(corrected_pos.x, corrected_pos.y, width, height)
}

/// Draw an exit arrow with a specified color.
fn draw_exit_arrow_colored(position: Vec2, direction: ExitDirection, grid_size: f32, color: Color) {
    let x = position.x;
    let y = position.y;

    let arrow_center = vec2(x + grid_size / 2.0, y + grid_size / 2.0);

    let offsets = match direction {
        ExitDirection::Up => [vec2(0.0, -1.0), vec2(-1.0, 1.0), vec2(1.0, 1.0)],
        ExitDirection::Down => [vec2(0.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, -1.0)],
        ExitDirection::Left => [vec2(-1.0, 0.0), vec2(1.0, -1.0), vec2(1.0, 1.0)],
        ExitDirection::Right => [vec2(1.0, 0.0), vec2(-1.0, -1.0), vec2(-1.0, 1.0)],
    };

    draw_triangle(
        arrow_center + offsets[0] * grid_size / 3.0,
        arrow_center + offsets[1] * grid_size / 3.0,
        arrow_center + offsets[2] * grid_size / 3.0,
        color,
    );
}