// engine_core/src/rendering/render_room.rs
use crate::{
    animation::animation_system::CurrentFrame, assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, camera::game_camera::RoomCamera, ecs::{
        component::*, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, global::tile_size, lighting::{
        glow::Glow, 
        light::Light,
    }, rendering::render_system::RenderSystem, tiles::tile::TileSprite, world::room::Room
};
use std::collections::{BTreeMap, HashMap};
use macroquad::prelude::*;

/// Draws everything needed for the given room.
pub fn render_room(
    world_ecs: &WorldEcs,
    room: &Room,
    asset_manager: &mut AssetManager,
    render_system: &mut RenderSystem,
    render_cam: &Camera2D,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>, 
) {
    // Cache the needed stores
    let sprite_store = world_ecs.get_store::<Sprite>();
    let frame_store = world_ecs.get_store::<CurrentFrame>();

    // Organize entities by layer
    let mut layer_map = collect_interpolated_layer_map(world_ecs, room, alpha, prev_positions);

    if layer_map.is_empty() {
        layer_map.insert(0, (Vec::new(), Vec::new()));
    }

    // Clear composite textures before each run
    RenderSystem::clear_cam(&render_system.scene_comp_rt);
    RenderSystem::clear_cam(&render_system.final_comp_rt);

    // Draw each blocking texture in black onto a white background
    // To be implemented but it needs to happen BEFORE the loop
    render_system.init_mask_cam();

    // Flag for the tilemap
    let mut first_pass = true;
    let tilemap = &room.current_variant().tilemap; 

    for (_z, (entities, glows)) in layer_map {
        // Scene cam needs to draw to the current render cam
        render_system.clear_scene_cam(render_cam);

        // Draw the tilemap as the first layer
        if first_pass {
            tilemap.draw(&room.exits, world_ecs, asset_manager, room.position);
            first_pass = false;
        }

        // Draw all entities
        for (entity, pos) in entities {
            draw_entity(
            world_ecs,
            asset_manager,
            frame_store,
            sprite_store,
            entity,
            pos,
            );
        }

        // Render the darkened scene
        render_system.run_ambient_pass(room.darkness);

        // Render glow per layer
        render_system.run_glow_pass(
            render_cam, 
            glows, 
            asset_manager,
        );

        // Render the undarkened room for lighting pass
        render_system.run_undarkened_pass();

        // Combine scene renders
        render_system.run_scene_pass();
    }

    // Lighting pass
    let lights = collect_lights(world_ecs, room, alpha, prev_positions);
    render_system.run_spotlight_pass(
        render_cam, 
        lights, 
        room.darkness
    );
    
    // Composite the final render
    render_system.run_final_pass();
}

fn draw_entity(
    world_ecs: &WorldEcs,
    asset_manager: &mut AssetManager,
    frame_store: &ComponentStore<CurrentFrame>,
    sprite_store: &ComponentStore<Sprite>,
    entity: Entity,
    pos: Vec2,
) {
    let (width, height) = entity_dimensions(world_ecs, asset_manager, entity);

    // Animate/Draw sprite
    if let Some(cf) = frame_store.get(entity) && asset_manager.contains(cf.sprite_id) {
        let src = Rect::new(
            cf.col as f32 * cf.frame_size.x,
            cf.row as f32 * cf.frame_size.y,
            cf.frame_size.x,
            cf.frame_size.y,
        );
        let tex = asset_manager.get_texture_from_id(cf.sprite_id);

        // Draws individual entites
        draw_texture_ex(
            tex,
            pos.x + cf.offset.x,
            pos.y + cf.offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                source: Some(src),
                ..Default::default()
            },
        );
        return;
    } else if let Some(sprite) = sprite_store.get(entity) {
        // No animation
        if asset_manager.contains(sprite.sprite_id) {
            
            let tex = asset_manager.get_texture_from_id(sprite.sprite_id);
            draw_texture_ex(
                tex,
                pos.x,
                pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    ..Default::default()
                },
            );
            return;
        }
    }

    // Don't draw placeholders for these components
    if world_ecs.has_any::<(Light, Glow)>(entity) {
        return;
    }

    // Fallback placeholder (no sprite or missing texture)
    draw_entity_placeholder(pos);
}

pub fn highlight_selected_entity(
    world_ecs: &WorldEcs,
    entity: Entity,
    asset_manager: &mut AssetManager,
    color: Color
) {
    let pos = match world_ecs.get_store::<Position>().get(entity) {
        Some(p) => p,
        None => return,
    };

    let (width, height) = entity_dimensions(world_ecs, asset_manager, entity);

    draw_rectangle_lines(pos.position.x, pos.position.y, width, height, 2.0, color);
}

pub fn entity_dimensions(
    world_ecs: &WorldEcs,
    asset_manager: &AssetManager,
    entity: Entity,
) -> (f32, f32) {
    let from_anim = world_ecs
        .get_store::<CurrentFrame>()
        .get(entity)
        .map(|cf| (cf.frame_size.x, cf.frame_size.y));

    let from_sprite = || {
        world_ecs
            .get_store::<Sprite>()
            .get(entity)
            .and_then(|spr| asset_manager.texture_size(spr.sprite_id))
    };

    let from_glow = || {
        world_ecs
            .get_store::<Glow>()
            .get(entity)
            .and_then(|glow| asset_manager.get_or_none(&glow.sprite_path))
            .and_then(|sprite_id| asset_manager.texture_size(sprite_id))
    };

    from_anim
        .or_else(from_sprite)
        .or_else(from_glow)
        .unwrap_or((tile_size(), tile_size()))
}

pub fn draw_entity_placeholder(pos: Vec2) {
    draw_rectangle(
        pos.x,
        pos.y,
        tile_size(),
        tile_size(),
        GREEN,
    );
}

/// Sorts entites by their z-layer, filters out entities that should not be 
/// drawn and interpolates the draw positions. BTreeMap automatically sorts keys.
fn collect_interpolated_layer_map<'a>(
    world_ecs: &'a WorldEcs,
    room: &Room,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
) -> BTreeMap<i32, (Vec<(Entity, Vec2)>, Vec<(&'a Glow, Vec2)>)> {
    let mut map: BTreeMap<i32, (Vec<(Entity, Vec2)>, Vec<(&Glow, Vec2)>)> = BTreeMap::new();

    let pos_store = world_ecs.get_store::<Position>();
    let tile_store = world_ecs.get_store::<TileSprite>();
    let cam_store = world_ecs.get_store::<RoomCamera>();
    let room_store = world_ecs.get_store::<CurrentRoom>();
    let layer_store = world_ecs.get_store::<Layer>();
    let glow_store = world_ecs.get_store::<Glow>();

    for (entity, pos) in &pos_store.data {
        // Skip tiles & camera
        if tile_store.get(*entity).is_some() || cam_store.get(*entity).is_some() {
            continue;
        }

        // Filter by current room
        if let Some(cr) = room_store.get(*entity) {
            if cr.0 != room.id { 
                continue; 
            }
        } else { 
            continue; 
        }

        // Interpolate the draw position
        let draw_pos = interpolate_draw_position(
            *entity, 
            pos.position, 
            alpha, 
            prev_positions
        );

        // Default layer is 0 if missing
        let z = layer_store
            .get(*entity)
            .map_or(0, |l| l.z);

        let entry = map.entry(z).or_default();
        entry.0.push((*entity, draw_pos));

        // If the entity also has a Glow component
        if let Some(glow) = glow_store.get(*entity) {
            entry.1.push((glow, draw_pos));
        }
    }

    // There always needs to be at least one layer otherwise nothing will be drawn
    if map.is_empty() {
        map.insert(0, (Vec::new(), Vec::new()));
    }

    map
}

fn collect_lights(
    world_ecs: &WorldEcs,
    room: &Room, 
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
) -> Vec<(Vec2, Light)> {
    let mut lights: Vec<(Vec2, Light)> = Vec::new();

    let light_store = world_ecs.get_store::<Light>();
    let pos_store = world_ecs.get_store::<Position>();
    let room_store = world_ecs.get_store::<CurrentRoom>();

    for (entity, light) in &light_store.data {
        // Filter by current room
        if let Some(current_room) = room_store.get(*entity) {
            if current_room.0 != room.id { 
                continue; 
            }
        } else { 
            continue; 
        }

        if let Some(pos) = pos_store.get(*entity) {
            // Interpolate the draw position
            let draw_pos = interpolate_draw_position(
                *entity, 
                pos.position, 
                alpha, 
                prev_positions
            );

            lights.push((draw_pos, *light));
        }
    }

    lights
}

/// Returns the interpolated draw position or the current position.
fn interpolate_draw_position(
    entity: Entity,
    current_pos: Vec2, 
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
) -> Vec2 {
    if let Some(prev_map) = prev_positions {
        if let Some(prev_pos) = prev_map.get(&entity) {
            lerp(*prev_pos, current_pos, alpha)
        }
        else {
            current_pos
        }
    } else {
        current_pos
    }
}

#[inline]
pub fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}