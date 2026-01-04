// engine_core/src/ecs/entity.rs
use crate::ecs::component_registry::ComponentRegistry;
use crate::ecs::component::*;
use crate::ecs::ecs::Ecs;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use inventory::iter;

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
    pub ecs: &'a mut Ecs,
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

        // Insert `T` and every component listed in the macro’s requirement list.
        (reg.factory)(self.ecs, self.id);
        T::store_mut(self.ecs).insert(self.id, comp);
        self
    }

    /// Finish the builder and get the public `Entity` back.
    pub fn finish(self) -> Entity {
        self.id
    }
}

