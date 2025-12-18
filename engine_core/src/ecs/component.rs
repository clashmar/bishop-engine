// engine_core/src/ecs/component.rs
use crate::assets::asset_manager::AssetManager;
use crate::ecs::world_ecs::WorldEcs;
use crate::world::room::RoomId;
use crate::ecs::entity::Entity;
use crate::inspector_module;
use ecs_component::ecs_component;
use reflect_derive::Reflect;
use std::{any::Any, collections::HashMap};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use macroquad::prelude::*;

/// Marker trait for components.
pub trait Component: Send + Sync {
    fn store_mut(world: &mut WorldEcs)
        -> &mut ComponentStore<Self>
    where
        Self: Sized;
        
    fn store(world: &WorldEcs)
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

/// Component bag that can remembers components for a entity and can restore them.
pub struct ComponentEntry {
    /// The concrete component value.
    pub value: Box<dyn Any>,
    /// Function that can clone the boxed value.
    pub cloner: fn(&dyn Any) -> Box<dyn Any>,
}

impl Clone for ComponentEntry {
    fn clone(&self) -> Self {
        Self {
            value: (self.cloner)(&*self.value),
            cloner: self.cloner,
        }
    }
}

/// Can be alled once a component has been added to an entity to initialize it.
pub trait PostCreate {
    fn post_create(
        &mut self,
        world_ecs: &mut WorldEcs,
        entity: Entity,
        asset_manager: &mut AssetManager,
    );
}

#[ecs_component]
#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Position {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
}

/// Z layer of an entity.
#[ecs_component]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, Reflect)]
#[serde(default)]
pub struct Layer {
    pub z: i32,
}
inspector_module!(Layer);

/// Component that stores the room identifier an entity belongs to.
#[ecs_component]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CurrentRoom(pub RoomId);

/// Marker component for the player entity.
#[ecs_component(deps = [Collider, Velocity])]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Player;

#[ecs_component]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, Reflect)]
#[serde(default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[ecs_component]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Grounded(#[serde(skip)] pub bool);

#[ecs_component]
#[derive(Clone, Copy, Serialize, Deserialize, Reflect)]
#[serde(default)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
}
inspector_module!(Collider);

impl Default for Collider {
    fn default() -> Self {
        Self {
            width:  16.0,
            height: 16.0,
        }
    }
}

/// Marker for participation in the physics system.
#[ecs_component(deps = [Grounded])]
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsBody;     

/// Marker for entities that move by code.
#[ecs_component]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Kinematic {}

// Tile components
#[ecs_component]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Walkable(pub bool);

#[ecs_component]
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Solid(pub bool);

#[ecs_component]
#[derive(Clone, Copy, Serialize, Deserialize, Default, Reflect)]
pub struct Damage {
    pub amount: f32,
}

