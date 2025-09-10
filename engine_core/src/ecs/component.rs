// engine_core/src/ecs/component.rs
use std::any::{Any, TypeId};
use reflect_derive::Reflect;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use macroquad::prelude::*;
use crate::ecs::entity::Entity; 

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

/// One entry for a concrete component type.
pub struct ComponentReg {
    /// Human‑readable identifier that will appear in the save file.
    pub type_name: &'static str,
    /// The concrete `ComponentStore<T>`’s `TypeId`.
    pub type_id: TypeId,
    /// Convert a concrete `ComponentStore<T>` (as a reference) into a `String`.
    pub to_ron: fn(&dyn Any) -> String,
    /// Convert a `String` back into a boxed concrete store.
    pub from_ron: fn(String) -> Box<dyn Any + Send>,
}
// Collect all registrations into a slice that lives for the whole program.
inventory::collect!(ComponentReg);

/// Register a component type and wire it into the dynamic store map.
#[macro_export]
macro_rules! ecs_component {
    ($ty:ty) => {
        impl $crate::ecs::component::Component for $ty {
            fn store_mut(
                world: &mut $crate::ecs::world_ecs::WorldEcs,
            ) -> &mut $crate::ecs::component::ComponentStore<Self> {
                world.get_or_create_store::<Self>()
            }
            fn store(
                world: &$crate::ecs::world_ecs::WorldEcs,
            ) -> &$crate::ecs::component::ComponentStore<Self> {
                world.get_store::<Self>()
            }
        }

        // Helpers for (de)serialization
        impl $ty {
            pub const TYPE_NAME: &'static str = stringify!($ty);

            fn to_ron(store: &dyn std::any::Any) -> String {
                let concrete = store
                    .downcast_ref::<$crate::ecs::component::ComponentStore<$ty>>()
                    .expect("type mismatch in to_ron");
                ron::ser::to_string_pretty(concrete, ron::ser::PrettyConfig::default())
                    .expect("failed to serialize ComponentStore")
            }

            fn from_ron(text: String) -> Box<dyn std::any::Any + Send> {
                let concrete: $crate::ecs::component::ComponentStore<$ty> =
                    ron::de::from_str(&text)
                        .expect("failed to deserialize ComponentStore");
                    Box::new(concrete)
            }
        }

        // Register in the global inventory
        inventory::submit! {
            $crate::ecs::component::ComponentReg {
                type_name: <$ty>::TYPE_NAME,
                type_id: std::any::TypeId::of::<$crate::ecs::component::ComponentStore<$ty>>(),
                to_ron: <$ty>::to_ron,
                from_ron: <$ty>::from_ron,
            }
        }
    };
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredComponent {
    pub type_name: String,
    pub data: String,
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

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
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