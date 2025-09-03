use crate::ecs::world_ecs::WorldEcs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::FromInto;
use macroquad::prelude::*;

use crate::ecs::entity::Entity; 

pub trait Component {
    fn store_mut(world: &mut WorldEcs) -> &mut ComponentStore<Self> where Self: Sized;
}

#[derive(Default, Serialize, Deserialize)]
pub struct ComponentStore<T> {
    pub data: HashMap<Entity, T>,
}

impl<T> ComponentStore<T> {
    pub fn insert(&mut self, entity: Entity, component: T) {
        self.data.insert(entity, component);
    }
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.data.get(&entity)
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.data.get_mut(&entity)
    }
    pub fn remove(&mut self, entity: Entity) {
        self.data.remove(&entity);
    }
}

macro_rules! ecs_component {
    ($type:ty, $field:ident) => {
        impl Component for $type {
            fn store_mut(world: &mut WorldEcs) -> &mut ComponentStore<Self> {
                &mut world.$field
            }
        }
    };
}

#[serde_as]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Position {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
}

ecs_component!(Position, positions);

#[serde_as]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Velocity {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub vel: Vec2,
}

ecs_component!(Velocity, velocities);

#[derive(Clone, Serialize, Deserialize)]
pub struct Animation {
    pub current: String, // e.g. "idle", "run"
    pub timer: f32,    // seconds elapsed in the current frame
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Walkable(pub bool);

ecs_component!(Walkable, walkables);

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Solid(pub bool);

ecs_component!(Solid, solids);

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Damage {
    pub amount: f32,
}

ecs_component!(Damage, damages);