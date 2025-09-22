// engine_core/src/physics/collider_system.rs
use crate::{
    assets::{asset_manager::AssetManager, sprite::{Sprite, SpriteId}},
    ecs::{component::Collider, entity::Entity, world_ecs::WorldEcs},
};

/// Set the collider for every entity that has a sprite and an unset collider
pub fn update_colliders_from_sprites(world_ecs: &mut WorldEcs, assets: &mut AssetManager) {
    let mut pending: Vec<(Entity, SpriteId)> = Vec::new();

    {
        // Immutable access to the two stores.
        let sprite_store   = world_ecs.get_store::<Sprite>();
        let collider_store = world_ecs.get_store::<Collider>();

        // Walk through every sprite
        for (entity, sprite) in sprite_store.data.iter() {
            if let Some(col) = collider_store.get(*entity) {
                // Unset colliders are recognised by a width/height of 0
                if col.width == 0.0 && col.height == 0.0 {
                    pending.push((*entity, sprite.sprite_id));
                }
            }
        }
    }

    // Mutate the Collider store
    if pending.is_empty() {
        return;
    }

    let collider_store = world_ecs.get_store_mut::<Collider>();

    for (entity, sprite_id) in pending {
        if let Some(collider) = collider_store.get_mut(entity) {
            *collider = collider_from_sprite(assets, sprite_id).unwrap_or_else(|| Collider {
                width: 0.0,
                height: 0.0,
            });
        }
    }
}

/// Returns a Collider whose dimensions match the sprite size.
pub fn collider_from_sprite(
    asset_manager: &mut AssetManager,
    sprite_id: SpriteId,
) -> Option<Collider> {
    asset_manager
        .texture_size(sprite_id)
        .map(|(w, h)| Collider { width: w, height: h })
}