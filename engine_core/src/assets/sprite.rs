// engine_core/src/assets/sprite.rs
use crate::game::game::GameCtxMut;
use crate::ecs::entity::Entity;
use crate::inspector_module;
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use reflect_derive::Reflect;


/// Opaque handle that the asset manager gives out. Default/Unset is 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SpriteId(pub usize);

#[ecs_component(post_create = post_create, post_remove = post_remove)]
#[derive(Clone, Serialize, Deserialize, Reflect)]
pub struct Sprite {
    /// Reference to the texture stored by the AssetManager.
    pub sprite: SpriteId,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            sprite: SpriteId(0),
        }
    }
}

fn post_create(sprite: &mut Sprite, _entity: &Entity, ctx: &mut GameCtxMut) {
    ctx.asset_manager.increment_ref(sprite.sprite);
}

fn post_remove(sprite: &mut Sprite, _entity: &Entity, ctx: &mut GameCtxMut) {
    ctx.asset_manager.decrement_ref(sprite.sprite);
}

inspector_module!(Sprite);