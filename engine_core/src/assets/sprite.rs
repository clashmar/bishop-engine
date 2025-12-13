// engine_core/src/assets/sprite.rs
use ecs_component::ecs_component;
use macroquad::prelude::*;
use reflect_derive::Reflect;
use serde::{Deserialize, Serialize};

use crate::inspector_module;

/// Opaque handle that the asset manager gives out. Default/Unset is 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SpriteId(pub usize);

#[ecs_component]
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

inspector_module!(Sprite);