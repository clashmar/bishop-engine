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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_tile"
    )]
    pub entity: Option<Entity>,
}

fn deserialize_tile<'de, D>(deserializer: D) -> Result<Option<Entity>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?)
}


#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TileSprite {
    pub sprite_id: SpriteId,
}

ecs_component!(TileSprite);

