// engine_core/src/rendering/render_entities.rs
use crate::{
    animation::animation_system::CurrentFrame, assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, 
    constants::*, 
    ecs::{
        component::*, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, 
    lighting::{
        glow::Glow, 
        light::Light, 
        light_system::LightSystem
    }, 
    tiles::tile::TileSprite, 
    world::room::Room
};
use std::collections::BTreeMap;
use macroquad::prelude::*;

/// Draws everything needed for the given room.
pub fn render_room(
    world_ecs: &WorldEcs,
    room: &Room,
    asset_manager: &mut AssetManager,
    lighting: &mut LightSystem,
    render_cam: &Camera2D,
) {
    // Cache the needed stores
    let sprite_store = world_ecs.get_store::<Sprite>();
    let frame_store = world_ecs.get_store::<CurrentFrame>();

    // Organize entities by layer
    let layer_map = collect_layer_map(world_ecs, room);

    LightSystem::clear_cam(&lighting.composite_rt);

    // Draw each blocking texture in black onto a white background
    // To be implemented but it needs to happen BEFORE the loop
    lighting.init_mask_cam();
    
    let darkness = 0.4f32; // TODO expose to editor

    // Flag for the tilemap
    let mut first_pass = true;
    let tilemap = &room.current_variant().tilemap; 

    for (_z, (entities, glows)) in layer_map {
        // Reset before using them
        lighting.clear_light_buffers();

        // Reset the cameras (except the composite cam)
        // Scene cam needs to draw to the current render cam
        let scene_cam = Camera2D {
            target: render_cam.target,
            zoom: render_cam.zoom,
            render_target: Some(lighting.scene_rt.clone()),
            ..Default::default()
        };

        set_camera(&scene_cam);

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
            *pos,
            );
        }

        // Glow pass
        lighting.run_glow_pass(render_cam, glows, asset_manager);
    }

    // Ambient pass
    lighting.run_ambient_pass(darkness);
        
    let lights = collect_lights(world_ecs, room);
    
    // Spotlight pass
    if !lights.is_empty() {
        lighting.run_spotlight_pass(
            render_cam, 
            lights, 
            darkness
        );
    }

    // Composite pass
    lighting.run_composite_pass();

    // Draw everything to the screen
    set_default_camera();
    draw_texture_ex(
        &lighting.composite_rt.texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams::default(),
    );
}

fn draw_entity(
    world_ecs: &WorldEcs,
    asset_manager: &mut AssetManager,
    frame_store: &ComponentStore<CurrentFrame>,
    sprite_store: &ComponentStore<Sprite>,
    entity: Entity,
    pos: Position,
) {
    let (width, height) = sprite_dimensions(world_ecs, asset_manager, entity);

    // Animate/Draw sprite
    if let Some(cf) = frame_store.get(entity) && asset_manager.contains(cf.sprite_id) {
        // Source rect = column/row * frame size
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
            pos.position.x + cf.offset.x,
            pos.position.y + cf.offset.y,
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
                pos.position.x,
                pos.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    ..Default::default()
                },
            );
            return;
        }
    }

    // Fallback placeholder (no sprite or missing texture)
    draw_entity_placeholder(pos.position);
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

    let (width, height) = sprite_dimensions(world_ecs, asset_manager, entity);

    draw_rectangle_lines(pos.position.x, pos.position.y, width, height, 2.0, color);
}

pub fn sprite_dimensions(
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

    from_anim.or_else(from_sprite).unwrap_or((TILE_SIZE, TILE_SIZE))
}

pub fn draw_entity_placeholder(pos: Vec2) {
    draw_rectangle(
        pos.x,
        pos.y,
        TILE_SIZE,
        TILE_SIZE,
        GREEN,
    );
}

/// Sorts entites by their z-layer and filters out entities that 
/// should not be drawn. BTreeMap automatically sorts keys.
fn collect_layer_map<'a>(
    world_ecs: &'a WorldEcs,
    room: &Room,
) -> BTreeMap<i32, (Vec<(Entity, &'a Position)>, Vec<(Vec2, &'a Glow)>)> {
    let mut map: BTreeMap<i32, (Vec<(Entity, &Position)>, Vec<(Vec2, &Glow)>)> = BTreeMap::new();

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

        // Default layer is 0 if missing
        let z = layer_store
            .get(*entity)
            .map_or(0, |l| l.z);

        let entry = map.entry(z).or_default();
        entry.0.push((*entity, pos));

        // If the entity also has a Glow component
        if let Some(l) = glow_store.get(*entity) {
            entry.1.push((pos.position, l));
        }
    }

    map
}

fn collect_lights(
    world_ecs: &WorldEcs,
    room: &Room, 
) -> Vec<(Vec2, Light)> {
    let mut lights: Vec<(Vec2, Light)> = Vec::new();

    let light_store = world_ecs.get_store::<Light>();
    let pos_store = world_ecs.get_store::<Position>();
    let room_store = world_ecs.get_store::<CurrentRoom>();

    for (entity, light) in &light_store.data {
        // Filter by current room
        if let Some(cr) = room_store.get(*entity) {
            if cr.0 != room.id { 
                continue; 
            }
        } else { 
            continue; 
        }

        if let Some(pos) = pos_store.get(*entity) {
            lights.push((pos.position, *light));
        }
    }
    lights
}

pub fn world_distance_to_screen(cam: &Camera2D, distance: f32) -> f32 {
    let scale = cam.zoom.x * screen_width() * 0.5; 
    (distance * scale).abs()
}