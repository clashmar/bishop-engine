// engine_core/src/ecs/component.rs
use reflect_derive::Reflect;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use macroquad::prelude::*;
use crate::{ecs::entity::Entity, ecs_component, inspector_module}; 

/// Marker trait - a component only needs to give access to its store.
pub trait Component: Send + Sync {
    fn store_mut(world: &mut crate::ecs::world_ecs::WorldEcs)
        -> &mut ComponentStore<Self>
    where
        Self: Sized;
        
    fn store(world: &crate::ecs::world_ecs::WorldEcs)
        -> &ComponentStore<Self>
    where
        Self: Sized;

}

#[derive(Serialize, Deserialize)]
pub struct ComponentStore<T> {
    pub data: HashMap<Entity, T>,
}

impl<T> Default for ComponentStore<T> {
    fn default() -> Self {
        ComponentStore {
            data: HashMap::new(),
        }
    }
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

    pub fn contains(&self, entity: Entity) -> bool {
        self.data.contains_key(&entity)
    }
}

#[serde_as]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Position {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
}

ecs_component!(Position);

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Walkable(pub bool);

ecs_component!(Walkable);

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Solid(pub bool);

ecs_component!(Solid);

#[derive(Clone, Copy, Serialize, Deserialize, Default, Reflect)]
pub struct Damage {
    pub amount: f32,
}

ecs_component!(Damage);

#[derive(Serialize, Deserialize, Default, Reflect)]
pub struct Weapon {
    pub name: String,
    pub damage: f32,
    pub range: f32,
    pub cooldown: f32,
}

ecs_component!(Weapon);
inspector_module!(Weapon);   