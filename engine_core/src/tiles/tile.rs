// engine_core/src/tiles/tile.rs
use crate::{
    assets::sprite::SpriteId, 
    ecs::entity::Entity, 
    ecs_component
};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Tile {
    pub entity: Entity,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TileSprite {
    #[serde(skip)] 
    pub sprite_id: SpriteId,
    pub path: String,
}

ecs_component!(TileSprite);

