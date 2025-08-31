use serde::{Deserialize, Serialize};
use macroquad::prelude::*;

use crate::ecs::{component::*, entity::{Entity, EntityBuilder}}; 

#[derive(Default, Serialize, Deserialize)]
pub struct WorldEcs {
    pub positions: ComponentStore<Position>,
    pub velocities: ComponentStore<Velocity>,
    pub sprites: ComponentStore<Sprite>,
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
}