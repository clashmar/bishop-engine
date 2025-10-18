// editor/src/room/room_editor_actions.rs
use engine_core::{
    animation::animation_clip::Animation, 
    assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, 
    ecs::{
        component::{Collider, Position, RoomCamera}, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, 
    global::tile_size, 
    lighting::{glow::Glow, light::Light}, 
    rendering::render_room::entity_dimensions, 
    world::room::Room
};
use crate::{editor_camera_controller::*, room::room_editor::RoomEditor};
use macroquad::prelude::*;
use crate::world::coord;

const PLACEHOLDER_OPACITY: f32 = 0.2;
fn thickness() -> f32 { (tile_size() * 0.175).max(1.0) }

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
pub fn draw_camera_placeholder(pos: Vec2) {
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

/// Draw an icon for a `Light` that has no other visual component.
pub fn draw_light_placeholders(world_ecs: &WorldEcs) {
    for (entity, _light) in world_ecs.get_store::<Light>().data.iter() {
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
pub fn draw_glow_placeholders(world_ecs: &WorldEcs, asset_manager: &mut AssetManager) {
    for (entity, glow) in world_ecs.get_store::<Glow>().data.iter() {
        if world_ecs.has_any::<(Sprite, Animation)>(*entity) {
            continue;
        }

        if let Some(position) = world_ecs.get_store::<Position>().get(*entity) {
            let mut pos = position.position;

            if let Some(sprite_id) = asset_manager.get_or_load(&glow.sprite_path) {
                if let Some((w, h)) = asset_manager.texture_size(sprite_id) {
                    pos = pos + vec2((w / 2.) - tile_size() / 2., (h / 2.) - tile_size() / 2.);
                }
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