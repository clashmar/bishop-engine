use crate::ecs::component::ComponentStore;
use crate::ecs::world_ecs::WorldEcs;
use crate::ecs::component::Component;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque handle that the asset manager gives out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SpriteId(pub Uuid);

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// Reference to the texture that was loaded by the AssetManager.
    pub tex_id: SpriteId,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            tex_id: SpriteId(Uuid::new_v4()),
        }
    }
}
impl Component for Sprite {
    fn store_mut(world: &mut WorldEcs) -> &mut ComponentStore<Self> {
        &mut world.sprites
    }
}