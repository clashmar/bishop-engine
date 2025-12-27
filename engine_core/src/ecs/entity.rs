// engine_core/src/ecs/entity.rs
use crate::ecs::component::{Component, ComponentStore, CurrentRoom};
use crate::ecs::component_registry::ComponentRegistry;
use crate::world::room::RoomId;  
use crate::ecs::ecs::Ecs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::any::TypeId;
use inventory::iter;

// TODO: Add name?
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct Entity(pub usize);

impl Entity {
    /// A sentinal value that can be used for optionals.
    pub fn null() -> Self {
        Entity(0)
    }
}

impl std::ops::Deref for Entity {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Entity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct EntityBuilder<'a> {
    pub id: Entity,
    pub world_ecs: &'a mut Ecs,
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
            // TODO handle expect
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

// Returns a HashSet of all entities in the current room.
pub fn entities_in_room(world_ecs: &mut Ecs, room_id: RoomId) -> HashSet<Entity> {
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

