use crate::{assets::sprites::SpriteId, ecs::{component::{Component, ComponentStore}, entity::Entity, world_ecs::WorldEcs}};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Tile {
    pub entity: Entity,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TileSprite {
    #[serde(skip)] 
    pub sprite: SpriteId,
    pub path: String,
}

impl Component for TileSprite {
    fn store_mut(world: &mut WorldEcs) -> &mut ComponentStore<Self> {
        &mut world.tile_sprites
    }
}

