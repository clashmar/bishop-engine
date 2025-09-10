// engine_core/src/ecs/entity.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::ecs::component::Component;
use crate::ecs::world_ecs::WorldEcs;  

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct Entity(pub Uuid);

impl Entity {
    /// Create a new UUID.
    pub fn new() -> Self {
        Entity(Uuid::new_v4())
    }

    /// A null value that can be used for optionals.
    pub fn null() -> Self {
        Entity(Uuid::nil())
    }
}

pub struct EntityBuilder<'a> {
    pub(crate) id:    Entity,
    pub world_ecs: &'a mut WorldEcs,
}

impl<'a> EntityBuilder<'a> {
    /// Attach any component that implements the `Component` marker trait.
    pub fn with<T>(self, comp: T) -> Self
    where
        T: Component,
    {
        T::store_mut(self.world_ecs).insert(self.id, comp);
        self
    }

    /// Finish the builder and get the public `Entity` back.
    pub fn finish(self) -> Entity {
        self.id
    }
}

