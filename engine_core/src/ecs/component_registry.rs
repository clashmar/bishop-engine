// engine_core/src/ecs/component_registry.rs
use crate::ecs::component::Component;
use once_cell::sync::Lazy;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use macroquad::prelude::*;
use crate::ecs::{entity::Entity, world_ecs::WorldEcs}; 

/// Human‑readable names of all components that have been registered with `ecs_component!`.
pub static COMPONENTS: Lazy<Vec<&'static ComponentReg>> = Lazy::new(|| {
    inventory::iter::<ComponentReg>.into_iter().collect()
});

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

    pub factory: fn(&mut WorldEcs, Entity),
    /// Returns true if the supplied entity already owns this component.
    pub has: fn(&WorldEcs, Entity) -> bool,
}

/// Factory that works for any component that implements `Component + Default`.
pub fn generic_factory<T>(world_ecs: &mut WorldEcs, entity: Entity)
where
    T: Component + Default + 'static,
{
    // Directly insert the default component into its typed store.
    world_ecs.get_store_mut::<T>().insert(entity, T::default());
}

pub fn has_component<T>(world: &WorldEcs, entity: Entity) -> bool
where
    T: Component + 'static,
{
    world.get_store::<T>().contains(entity)
}

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
            $crate::ecs::component_registry::ComponentReg {
                type_name: <$ty>::TYPE_NAME,
                type_id: std::any::TypeId::of::<
                    $crate::ecs::component::ComponentStore<$ty>
                >(),
                to_ron: <$ty>::to_ron,
                from_ron: <$ty>::from_ron,
                factory: $crate::ecs::component_registry::generic_factory::<$ty>,
                has: $crate::ecs::component_registry::has_component::<$ty>,
            }
        }
    };
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredComponent {
    pub type_name: String,
    pub data: String,
}

// Collect all registrations into a slice that lives for the whole program.
inventory::collect!(ComponentReg);