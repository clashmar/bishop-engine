use std::collections::HashMap;

use crate::{
    assets::sprites::Sprite, 
    ecs::{component::*, entity::{Entity, EntityBuilder}}, 
    tiles::{tile::TileSprite, tile_def::{TileDef, TileDefId}}
}; 
use serde::{Deserialize, Serialize};
use macroquad::prelude::*;

#[derive(Default, Serialize, Deserialize)]
pub struct WorldEcs {
    pub positions: ComponentStore<Position>,
    pub velocities: ComponentStore<Velocity>,
    pub sprites: ComponentStore<Sprite>,
    pub walkables: ComponentStore<Walkable>,
    pub solids: ComponentStore<Solid>,
    pub damages: ComponentStore<Damage>,
    pub tile_defs: HashMap<TileDefId, TileDef>,
    pub tile_sprites: ComponentStore<TileSprite>,
    component: (),
}

impl WorldEcs {
    /// Allocate a fresh UUID and return a builder.
    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder {
            id: Entity::new(),
            world: self,
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.positions.remove(entity);
        self.velocities.remove(entity);
        self.sprites.remove(entity);
        self.walkables.remove(entity);
        self.solids.remove(entity);
        self.damages.remove(entity);
        self.tile_sprites.remove(entity);
    }
}