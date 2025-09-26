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
use macroquad::prelude::*;

pub fn draw_entities(
    world_ecs: &WorldEcs,
    room: &Room,
    asset_manager: &mut AssetManager,
) {
    // Cache the stores – no extra hashmap look‑ups inside the loop
    let pos_store = world_ecs.get_store::<Position>();
    let tile_store = world_ecs.get_store::<TileSprite>();
    let room_store = world_ecs.get_store::<CurrentRoom>();
    let sprite_store = world_ecs.get_store::<Sprite>();
    let camera_store = world_ecs.get_store::<RoomCamera>();
    let frame_store = world_ecs.get_store::<CurrentFrame>();

    for (entity, pos) in pos_store.data.iter() {
        // Skip tiles and camera
        if tile_store.get(*entity).is_some() || camera_store.get(*entity).is_some() {
            continue;
        }

        // Draw only if the entity belongs to the current room
        if let Some(current_room) = room_store.get(*entity) {
            if current_room.0 != room.id {
                continue;
            }
        } else {
            continue;
        }

        // Animate/Draw sprite
        if let Some(cf) = frame_store.get(*entity) && asset_manager.contains(cf.sprite_id) {
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
            continue; // Frame drawn, go to next entity
        } else if let Some(sprite) = sprite_store.get(*entity) {
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
                continue; // sprite drawn, go to next entity
            }
        }

        // Fallback placeholder (no sprite or missing texture)
        draw_entity_placeholder(pos.position);
    }
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
    if let Some(sprite) = world_ecs.get_store::<Sprite>().get(entity) {
        if let Some((width, height)) = asset_manager.texture_size(sprite.sprite_id) {
            return (width, height);
        }
    }
    // Fallback
    (TILE_SIZE, TILE_SIZE)
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