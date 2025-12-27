// editor/src/room/room_editor_actions.rs
use crate::editor_camera_controller::*;
use crate::gui::gui_constants::*;
use crate::room::room_editor::*;
use crate::gui::menu_bar::*;
use crate::world::coord;
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
use engine_core::engine_global::*;
use engine_core::ui::widgets::*;
use engine_core::world::room::*;
use engine_core::ecs::ecs::Ecs;
use engine_core::ui::text::*;
use macroquad::prelude::*;

const PLACEHOLDER_OPACITY: f32 = 0.2;
fn thickness() -> f32 { (tile_size() * 0.175).max(1.0) }

impl RoomEditor {
    /// Draw static UI for the scene editor
    pub fn draw_ui(
        &mut self, 
        game_ctx: &mut GameCtxMut,
    ) {
        // Reset to static camera
        set_default_camera();

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
    pub fn draw_coordinates(&self, camera: &Camera2D, room: &Room) {
        let local_grid = coord::mouse_world_grid(camera);
        let world_grid = local_grid + room.position;
        
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

    /// Draw a yellow rectangle that visualises the viewport of a selected RoomCamera.
    pub fn draw_camera_viewport(
        &self,
        editor_cam: &Camera2D,
        world_ecs: &Ecs,
        selected: Entity,
    ) {
        let pos = match world_ecs.get_store::<Position>().get(selected) {
            Some(p) => p.position,
            None => return,
        };

        let room_cam = match world_ecs.get_store::<RoomCamera>().get(selected) {
            Some(c) => c,
            None => return,
        };

        let factor_x = editor_cam.zoom.x / room_cam.zoom.x;
        let factor_y = editor_cam.zoom.y / room_cam.zoom.y;

        let bl = editor_cam.screen_to_world(vec2(0.0, 0.0));
        let tr = editor_cam.screen_to_world(vec2(screen_width(), screen_height()));
        let editor_w = (tr.x - bl.x).abs();
        let editor_h = (tr.y - bl.y).abs();

        let viewport_w = editor_w * factor_x;
        let viewport_h = editor_h * factor_y;

        let half = vec2(viewport_w, viewport_h) * 0.5;
        let top_left = pos - half;

        let editor_scalar = EditorCameraController::scalar_zoom(editor_cam);
        const BASE_THICKNESS: f32 = 1.;
        let thickness = BASE_THICKNESS * (MAX_ZOOM / editor_scalar).max(1.0);

        draw_rectangle_lines(
            top_left.x,
            top_left.y,
            viewport_w,
            viewport_h,
            thickness,
            YELLOW,
        );
    }
}

/// Draw the outline of the collider for an entity if it has one.
pub fn draw_collider(
    world_ecs: &Ecs,
    entity: Entity,
) {
    if let Some((width, height)) = world_ecs
        .get_store::<Collider>()
        .get(entity)
        .filter(|c| c.width > 0.0 && c.height > 0.0)
        .map(|c| (c.width, c.height)) {
            let pos = match world_ecs.get_store::<Position>().get(entity) {
                Some(p) => p.position,
                None => return,
            };

            draw_rectangle_lines(pos.x, pos.y, width, height, 2.0, PINK);
     }
}

/// Returns a `Rect` hitbox for an entity based on its sprite if it has one,
/// otherwise it returns a hitbox based on the default sprite dimensions.
pub fn entity_hitbox(
    entity: Entity,
    position: Vec2,
    camera: &Camera2D,
    world_ecs: &Ecs,
    asset_manager: &mut AssetManager,
) -> Rect {
    let (width, height) = entity_dimensions(world_ecs, asset_manager, entity);

    // If this is a camera or light, move the position from the top left
    // corner to the visual centre to match how it's drawn
    let corrected_pos = if world_ecs.has_any::<(RoomCamera, Light)>(entity) {
        position - vec2(tile_size() * 0.5, tile_size() * 0.5)
    } else {
        position
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
pub fn draw_camera_placeholders(world_ecs: &Ecs, room_id: RoomId) {
    let cam_store = world_ecs.get_store::<RoomCamera>();
    let pos_store = world_ecs.get_store::<Position>();
    let room_store = world_ecs.get_store::<CurrentRoom>();

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
        let half_tile = tile_size() * 0.5;
        let body = Rect::new(
            pos.x - half_tile,   
            pos.y - half_tile,
            tile_size(),
            tile_size(),
        );

        let green = Color::new(0.0, 0.89, 0.19, PLACEHOLDER_OPACITY);
        let blue = Color::new(0.0, 0.47, 0.95, PLACEHOLDER_OPACITY);
        let red = Color::new(0.9, 0.16, 0.22, PLACEHOLDER_OPACITY);

        draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness(), green);

        let finder_w = tile_size() * 0.3;
        let finder_h = tile_size() * 0.6;
        let finder = Rect::new(
            body.x + thickness(),                     
            body.y + (body.h - finder_h) / 2.0,
            finder_w,
            finder_h,
        );
        draw_rectangle_lines(finder.x, finder.y, finder.w, finder.h,
                            thickness() * 0.75, blue);

        let lens_radius = tile_size() * 0.1;
        let lens_center = vec2(
            body.x + body.w - lens_radius * 2.0 - thickness(),
            body.y + body.h / 2.0,
        );
        draw_circle_lines(lens_center.x, lens_center.y,
                        lens_radius, thickness() * 0.75, red);
        }
}

/// Draw an icon for a `Light` that has no other visual component.
pub fn draw_light_placeholders(
    world_ecs: &Ecs,
    room_id: RoomId,
) {
    let room_store = world_ecs.get_store::<CurrentRoom>();
    for (entity, _light) in world_ecs.get_store::<Light>().data.iter() {
        // Only draw placeholders in this room
        if let Some(CurrentRoom(id)) = room_store.get(*entity) {
            if *id != room_id { continue; }
        }

        // Don't draw if there is a Sprite or Animation component
        if world_ecs.has_any::<(Sprite, Animation)>(*entity) {
            continue;
        }

        if let Some(position) = world_ecs.get_store::<Position>().get(*entity) {
            let pos = position.position;

            let half_tile = tile_size() * 0.5;
            let body = Rect::new(
                pos.x - half_tile,
                pos.y - half_tile,
                tile_size(),
                tile_size(),
            );

            let cyan = Color::new(0.0, 0.78, 0.78, PLACEHOLDER_OPACITY);
            let yellow = Color::new(0.94, 0.86, 0.0, PLACEHOLDER_OPACITY);

            // Outer square
            draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness(), cyan);

            // Lens
            let lens_radius = tile_size() * 0.2;
            let lens_center = vec2(
                body.x + body.w / 2.,
                body.y + body.h / 2.,
            );

            draw_circle_lines(
                lens_center.x,
                lens_center.y,
                lens_radius,
                thickness() * 0.75,
                yellow,
            );
        }
    }
}

/// Draw a placeholder for a `Glow` that has no other visual component.
pub fn draw_glow_placeholders(
    world_ecs: &Ecs, 
    asset_manager: &mut AssetManager,
    room_id: RoomId,
) {
    let room_store = world_ecs.get_store::<CurrentRoom>();
    for (entity, glow) in world_ecs.get_store::<Glow>().data.iter() {
        // Only draw placeholders in this room
        if let Some(CurrentRoom(id)) = room_store.get(*entity) {
            if *id != room_id { continue; }
        }

        // Don't draw if there is a Sprite or Animation component
        if world_ecs.has_any::<(Sprite, Animation)>(*entity) {
            continue;
        }

        if let Some(position) = world_ecs.get_store::<Position>().get(*entity) {
            let mut pos = position.position;

            if let Some((w, h)) = asset_manager.texture_size(glow.sprite_id) {
                pos = pos + vec2((w / 2.) - tile_size() / 2., (h / 2.) - tile_size() / 2.);
            }

            let body = Rect::new(
                pos.x,
                pos.y,
                tile_size(),
                tile_size(),
            );

            let cyan = Color::new(0.0, 0.78, 0.78, PLACEHOLDER_OPACITY);
            let yellow = Color::new(0.94, 0.86, 0.0, PLACEHOLDER_OPACITY);

            // Outer square
            draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness(), cyan);

            // Lens
            let lens_radius = tile_size() * 0.2;
            let lens_center = vec2(
                body.x + body.w / 2.,
                body.y + body.h / 2.,
            );

            draw_circle_lines(
                lens_center.x,
                lens_center.y,
                lens_radius,
                thickness() * 0.75,
                yellow,
            );
        }
    }
}