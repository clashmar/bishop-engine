// engine_core/src/rendering/render_entities.rs
use crate::{
    animation::animation_system::CurrentFrame, assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, constants::*, ecs::{
        component::*, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, tiles::tile::TileSprite, world::room::Room
};
use std::collections::BTreeMap;
use macroquad::prelude::*;

pub fn draw_entities(
    world_ecs: &WorldEcs,
    room: &Room,
    asset_manager: &mut AssetManager,
) {
    let layers = collect_layer_map(world_ecs, room);

    // Cache the stores
    let sprite_store = world_ecs.get_store::<Sprite>();
    let frame_store = world_ecs.get_store::<CurrentFrame>();

    for (_z, entities) in layers {
        for (entity, pos) in entities {
            draw_entity(
            asset_manager,
            frame_store,
            sprite_store,
            entity,
            *pos,
            )
        }
    }
}

fn draw_entity(
    asset_manager: &mut AssetManager,
    frame_store: &ComponentStore<CurrentFrame>,
    sprite_store: &ComponentStore<Sprite>,
    entity: Entity,
    pos: Position,
) {
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
        draw_texture_ex(
            tex,
            pos.position.x + cf.offset.x,
            pos.position.y + cf.offset.y,
            WHITE,
            DrawTextureParams {
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
) -> BTreeMap<i32, Vec<(Entity, &'a Position)>> {
    let mut map: BTreeMap<i32, Vec<(Entity, &Position)>> = BTreeMap::new();

    let pos_store = world_ecs.get_store::<Position>();
    let tile_store = world_ecs.get_store::<TileSprite>();
    let cam_store = world_ecs.get_store::<RoomCamera>();
    let room_store = world_ecs.get_store::<CurrentRoom>();
    let layer_store = world_ecs.get_store::<Layer>();

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

        map.entry(z)
            .or_default()
            .push((*entity, pos));
    }

    map
}