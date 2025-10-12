// editor/src/room/room_editor_actions.rs
use engine_core::{
    assets::asset_manager::AssetManager, 
    camera::game_camera::zoom_from_scalar, 
    constants::*, 
    ecs::{
        component::{Collider, Position, RoomCamera}, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, 
    rendering::render_room::sprite_dimensions, 
    world::room::Room
};
use crate::{editor_camera_controller::*, room::room_editor::RoomEditor};
use macroquad::prelude::*;
use crate::world::coord;

impl RoomEditor {
    /// Draw the cursor coordinates in world space.
    pub fn draw_coordinates(&self, camera: &Camera2D, room: &Room) {
        let local_grid = coord::mouse_world_grid(camera);

        let world_grid = local_grid + room.position;
        
        let txt = format!(
            "({:.0}, {:.0})",
            world_grid.x, world_grid.y,
        );

        let margin = 10.0;
        draw_text(&txt, margin, screen_height() - margin, 20.0, BLUE);
    }

    /// Draw a yellow rectangle that visualises the viewport of a selected RoomCamera.
    pub fn draw_camera_viewport(
        &self,
        editor_cam: &Camera2D,
        world_ecs: &WorldEcs,
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

        let room_zoom = zoom_from_scalar(room_cam.scalar_zoom);

        let factor_x = editor_cam.zoom.x / room_zoom.x;
        let factor_y = editor_cam.zoom.y / room_zoom.y;

        let bl = editor_cam.screen_to_world(vec2(0.0, 0.0));
        let tr = editor_cam.screen_to_world(vec2(screen_width(), screen_height()));
        let editor_w = (tr.x - bl.x).abs();
        let editor_h = (tr.y - bl.y).abs();

        let viewport_w = editor_w * factor_x;
        let viewport_h = editor_h * factor_y;

        let half = vec2(viewport_w, viewport_h) * 0.5;
        let top_left = pos - half;

        let editor_scalar = EditorCameraController::scalar_zoom(editor_cam);
        const BASE_THICKNESS: f32 = 3.0;
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

/// Draw an icon for the `RoomCamera`.
pub fn draw_camera_placeholder(pos: Vec2) {
    // Offset the camera placeholder 
    let half_tile = TILE_SIZE * 0.5;
    let body = Rect::new(
        pos.x - half_tile,   
        pos.y - half_tile,
        TILE_SIZE,
        TILE_SIZE,
    );

    let thickness = (TILE_SIZE * 0.2).max(1.0);
    let green = Color::new(0.0, 0.89, 0.19, 0.5);
    let blue  = Color::new(0.0, 0.47, 0.95, 0.5);
    let red   = Color::new(0.9, 0.16, 0.22, 0.5);

    draw_rectangle_lines(body.x, body.y, body.w, body.h, thickness, green);

    let finder_w = TILE_SIZE * 0.3;
    let finder_h = TILE_SIZE * 0.6;
    let finder = Rect::new(
        body.x + thickness,                     
        body.y + (body.h - finder_h) / 2.0,
        finder_w,
        finder_h,
    );
    draw_rectangle_lines(finder.x, finder.y, finder.w, finder.h,
                         thickness * 0.75, blue);

    let lens_radius = TILE_SIZE * 0.1;
    let lens_center = vec2(
        body.x + body.w - lens_radius * 2.0 - thickness,
        body.y + body.h / 2.0,
    );
    draw_circle_lines(lens_center.x, lens_center.y,
                      lens_radius, thickness * 0.75, red);
}

/// Draw the outline of the collider for an entity if it has one.
pub fn draw_collider(
    world_ecs: &WorldEcs,
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
    world_ecs: &WorldEcs,
    asset_manager: &mut AssetManager,
) -> Rect {
    let (width, height) = sprite_dimensions(world_ecs, asset_manager, entity);

    // If this is a camera, move the position from the top left
    // corner to the visual centre to match how it's drawn
    let corrected_pos = if world_ecs.get_store::<RoomCamera>().get(entity).is_some() {
        position - vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5)
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