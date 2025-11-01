// engine_core/src/ecs/entity.rs
use std::any::TypeId;
use std::collections::HashSet;
use inventory::iter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::ecs::component::{Component, ComponentStore, CurrentRoom};
use crate::ecs::component_registry::ComponentRegistry;
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
    pub id: Entity,
    pub world_ecs: &'a mut WorldEcs,
}

impl<'a> EntityBuilder<'a> {
    /// Attach any component that implements the `Component` marker trait.
    pub fn with<T>(self, comp: T) -> Self
    where
        T: Component + Default + 'static,
    {
        // Find the registration entry for `T`.
        let reg = iter::<ComponentRegistry>()
            .find(|r| r.type_id == TypeId::of::<ComponentStore<T>>())
            .expect("Component not registered.");

        // Run the factory. This inserts `T` and every
        // component listed in the macro’s requirement list.
        (reg.factory)(self.world_ecs, self.id);

        T::store_mut(self.world_ecs).insert(self.id, comp);

        self
    }

    /// Finish the builder and get the public `Entity` back.
    pub fn finish(self) -> Entity {
        self.id
    }
}

pub fn entities_in_room(world_ecs: &mut WorldEcs, room_id: Uuid) -> HashSet<Entity> {
    let room_store = world_ecs.get_store::<CurrentRoom>();
    room_store
        .data
        .iter()
        .filter_map(|(entity, cur_room)| {
            if cur_room.0 == room_id {
                Some(*entity)
            } else {
                None
            }
        })
        .collect()
}

