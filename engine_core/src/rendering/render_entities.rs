// engine_core/src/rendering/render_entities.rs
use crate::{
    assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, 
    constants::*, 
    ecs::{component::{CurrentRoom, Position}, entity::Entity, world_ecs::WorldEcs
}, 
    tiles::tile::TileSprite, 
    world::room::Room
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

    for (entity, pos) in pos_store.data.iter() {
        // Skip tiles
        if tile_store.get(*entity).is_some() {
            continue;
        }

        // Draw only if the entity belongs to the current room
        if let Some(cur) = room_store.get(*entity) {
            if cur.0 != room.id {
                continue;
            }
        } else {
            continue;
        }

        // Position relative to the room origin
        let room_pos = pos.position - room.position;

        // Sprite handling – one branch instead of three
        if let Some(sprite) = sprite_store.get(*entity) {
            if asset_manager.contains(sprite.sprite_id) {
                let tex = asset_manager.get_texture_from_id(sprite.sprite_id);
                draw_texture_ex(
                    tex,
                    room_pos.x - TILE_SIZE / 2.0,
                    room_pos.y - TILE_SIZE / 2.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                );
                continue; // sprite drawn, go to next entity
            }
        }
        // Fallback placeholder (no sprite or missing texture)
        draw_entity_placeholder(room_pos);
    }
}

pub fn highlight_selected_entity(
    world_ecs: &WorldEcs,
    room: &Room,
    entity: Entity,
) {
    if let Some(pos) = world_ecs.get_store::<Position>().get(entity) {
        draw_rectangle_lines(
            pos.position.x - room.position.x - 11.0,
            pos.position.y - room.position.y - 11.0,
            22.0,
            22.0,
            2.0,
            YELLOW,
        );
    }
}

pub fn draw_entity_placeholder(pos: Vec2) {
    draw_rectangle(
        pos.x - 10.0,
        pos.y - 10.0,
        20.0,
        20.0,
        MAGENTA,
    );
}