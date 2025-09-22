// engine_core/src/rendering/render_entities.rs
use crate::{
    assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, 
    constants::*, 
    ecs::{
        component::*, 
        entity::{self, Entity}, 
        world_ecs::WorldEcs
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
    let camera_store = world_ecs.get_store::<RoomCamera>();

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

        // Position relative to the room origin
        let room_pos = pos.position - room.position;

        // Sprite handling
        if let Some(sprite) = sprite_store.get(*entity) {
            if asset_manager.contains(sprite.sprite_id) {
                let tex = asset_manager.get_texture_from_id(sprite.sprite_id);
                draw_texture_ex(
                    tex,
                    room_pos.x - TILE_SIZE / 2.0,
                    room_pos.y - TILE_SIZE / 2.0,
                    WHITE,
                    DrawTextureParams {
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
    asset_manager: &mut AssetManager,
) {
    let pos = match world_ecs.get_store::<Position>().get(entity) {
        Some(p) => p,
        None => return,
    };

    let (width, height, colour) = resolve_highlight_outline(
        world_ecs,
        asset_manager,
        entity,
    );

    let room_pos = pos.position - room.position;
    let x = room_pos.x - TILE_SIZE / 2.0;
    let y = room_pos.y - TILE_SIZE / 2.0;

    draw_rectangle_lines(x, y, width, height, 2.0, colour);
}

fn resolve_highlight_outline(
    world_ecs: &WorldEcs,
    asset_manager: &AssetManager,
    entity: Entity,
) -> (f32, f32, Color) {
    let mut width  = TILE_SIZE;
    let mut height = TILE_SIZE;

    // Try collider
    if let Some(col) = world_ecs.get_store::<Collider>().get(entity) {
        if col.width > 0.0 && col.height > 0.0 {
            return (col.width, col.height, PINK);
        }
    }

    // Try sprite
    if let Some(sprite) = world_ecs.get_store::<Sprite>().get(entity) {
        if let Some((w, h)) = asset_manager.texture_size(sprite.sprite_id) {
            width  = w;
            height = h;
        }
    }

    (width, height, YELLOW)
}

pub fn draw_entity_placeholder(pos: Vec2) {
    draw_rectangle(
        pos.x - 10.0,
        pos.y - 10.0,
        20.0,
        20.0,
        GREEN,
    );
}