// engine_core/src/assets/sprite.rs
use crate::ecs_component;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque handle that the asset manager gives out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SpriteId(pub Uuid);

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// Reference to the texture that was loaded by the AssetManager.
    pub sprite_id: SpriteId,
    pub path: String,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            sprite_id: SpriteId(Uuid::new_v4()),
            path: String::new(),
        }
    }
}

ecs_component!(Sprite);