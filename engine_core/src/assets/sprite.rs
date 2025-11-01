// engine_core/src/assets/sprite.rs
use crate::ecs_component;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

/// Opaque handle that the asset manager gives out. Default/Unset is 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SpriteId(pub usize);

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// Reference to the texture stored by the AssetManager.
    pub sprite_id: SpriteId,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            sprite_id: SpriteId(0),
        }
    }
}

ecs_component!(Sprite);