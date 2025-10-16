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
    pub from_ron: fn(String) -> Box<dyn Any + Send + Sync>,
    /// Factory that creates the component (and its dependencies) for an entity.
    pub factory: fn(&mut WorldEcs, Entity),
    /// Returns true if the supplied entity already owns this component.
    pub has: fn(&WorldEcs, Entity) -> bool,
    // Removes the component for `entity` from the concrete store.
    pub remove: fn(&mut WorldEcs, Entity),
    /// Function that knows how to write a boxed component back into the world.
    pub inserter: fn(&mut WorldEcs, Entity, Box<dyn Any>),
    /// Clones the concrete component for `entity` and returns it boxed as `dyn Any`.
    pub clone: fn(&WorldEcs, Entity) -> Box<dyn Any>,
    /// Serialize a single component.
    pub to_ron_component: fn(&dyn Any) -> String,
    /// Deserialize a single component.
    pub from_ron_component: fn(String) -> Box<dyn Any>,
    /// Called for optional run post‑create logic.  If `None` the engine will do nothing.
    pub post_create: fn(&mut dyn Any),
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

/// Helper that erases an entity from a concrete `ComponentStore<T>`.
pub fn erase_from_store<T>(world_ecs: &mut WorldEcs, entity: Entity)
where
    T: Component + 'static,
{
    world_ecs.get_store_mut::<T>().remove(entity);
}

/// Inserts a concrete component that has been boxed as `dyn Any`.
pub fn generic_inserter<T>(world_ecs: &mut WorldEcs, entity: Entity, boxed: Box<dyn Any>)
where
    T: Component + 'static,
{
    let concrete = *boxed
        .downcast::<T>()
        .expect("ComponentEntry contains wrong type");
    world_ecs.get_store_mut::<T>().insert(entity, concrete);
}

/// Register a component type and wire it into the dynamic store map.
#[macro_export]
macro_rules! ecs_component {
    // No requirements, no custom post_create
    ($ty:ty) => {
        $crate::ecs_component!(@final $ty, [], default);
    };
    // Requirements, no custom post_create
    ($ty:ty, [$($req:ty),* $(,)?]) => {
        $crate::ecs_component!(@final $ty, [$($req),*], default);
    };
    // No requirements, custom post_create
    ($ty:ty, post_create = $func:path) => {
        $crate::ecs_component!(@final $ty, [], custom $func);
    };
    // Requirements and custom post_create
    ($ty:ty, [$($req:ty),* $(,)?], post_create = $func:path) => {
        $crate::ecs_component!(@final $ty, [$($req),*], custom $func);
    };

    // Internal entry point
    (@final $ty:ty, [$($req:ty),*], default) => {
        // Implement Component trait for the concrete type
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

        impl $ty
        where
            $ty: 'static + Clone,
        {
            pub const TYPE_NAME: &'static str = stringify!($ty);

            fn __factory(
                world: &mut $crate::ecs::world_ecs::WorldEcs,
                entity: $crate::ecs::entity::Entity,
            ) {
                world.get_store_mut::<$ty>()
                    .insert(entity, <$ty>::default());
                $(
                    world.get_store_mut::<$req>()
                        .insert(entity, <$req>::default());
                )*
            }

            // (De)serialisation of the whole ComponentStore<$ty>
            fn __to_ron(store: &dyn std::any::Any) -> String {
                let concrete = store
                    .downcast_ref::<$crate::ecs::component::ComponentStore<$ty>>()
                    .expect("type mismatch in to_ron");
                ron::ser::to_string_pretty(concrete, ron::ser::PrettyConfig::default())
                    .expect("failed to serialize ComponentStore")
            }
            fn __from_ron(text: String) -> Box<dyn std::any::Any + Send + Sync> {
                let concrete: $crate::ecs::component::ComponentStore<$ty> =
                    ron::de::from_str(&text).expect("failed to deserialize ComponentStore");
                Box::new(concrete)
            }

            // (De)serialisation of a single component instance
            fn __to_ron_component(value: &dyn std::any::Any) -> String {
                let concrete = value
                    .downcast_ref::<$ty>()
                    .expect("type mismatch in to_ron_component");
                ron::ser::to_string_pretty(concrete, ron::ser::PrettyConfig::default())
                    .expect("failed to serialize component")
            }
            fn __from_ron_component(text: String) -> Box<dyn std::any::Any> {
                let concrete: $ty =
                    ron::de::from_str(&text).expect("failed to deserialize component");
                Box::new(concrete) as Box<dyn std::any::Any>
            }
        }

        // Register the component (default path)
        inventory::submit! {
            $crate::ecs::component_registry::ComponentReg {
                type_name: <$ty>::TYPE_NAME,
                type_id: std::any::TypeId::of::<
                    $crate::ecs::component::ComponentStore<$ty>
                >(),
                to_ron: <$ty>::__to_ron,
                from_ron: <$ty>::__from_ron,
                factory: <$ty>::__factory,
                has: $crate::ecs::component_registry::has_component::<$ty>,
                remove: $crate::ecs::component_registry::erase_from_store::<$ty>,
                inserter: $crate::ecs::component_registry::generic_inserter::<$ty>,
                clone: |world: &$crate::ecs::world_ecs::WorldEcs,
                         entity: $crate::ecs::entity::Entity| {
                    let store_any = world
                        .stores
                        .get(&std::any::TypeId::of::<
                            $crate::ecs::component::ComponentStore<$ty>
                        >())
                        .expect("store missing despite has() == true");
                    let component = {
                        let store = store_any
                            .downcast_ref::<
                                $crate::ecs::component::ComponentStore<$ty>
                            >()
                            .expect("type mismatch in store");
                        store
                            .get(entity)
                            .expect("has() returned true but component missing")
                            .clone()
                    };
                    Box::new(component) as Box<dyn std::any::Any>
                },
                to_ron_component: <$ty>::__to_ron_component,
                from_ron_component: <$ty>::__from_ron_component,
                post_create: $crate::ecs::component_registry::post_create,
            }
        }
    };

    // Custom post-create branch
    (@final $ty:ty, [$($req:ty),*], custom $func:path) => {
        // Implement Component trait for the concrete type
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

        impl $ty
        where
            $ty: 'static + Clone,
        {
            pub const TYPE_NAME: &'static str = stringify!($ty);

            fn __factory(
                world: &mut $crate::ecs::world_ecs::WorldEcs,
                entity: $crate::ecs::entity::Entity,
            ) {
                world.get_store_mut::<$ty>()
                    .insert(entity, <$ty>::default());
                $(
                    world.get_store_mut::<$req>()
                        .insert(entity, <$req>::default());
                )*
            }

            fn __to_ron(store: &dyn std::any::Any) -> String {
                let concrete = store
                    .downcast_ref::<$crate::ecs::component::ComponentStore<$ty>>()
                    .expect("type mismatch in to_ron");
                ron::ser::to_string_pretty(concrete, ron::ser::PrettyConfig::default())
                    .expect("failed to serialize ComponentStore")
            }
            fn __from_ron(text: String) -> Box<dyn std::any::Any + Send + Sync> {
                let concrete: $crate::ecs::component::ComponentStore<$ty> =
                    ron::de::from_str(&text).expect("failed to deserialize ComponentStore");
                Box::new(concrete)
            }

            fn __to_ron_component(value: &dyn std::any::Any) -> String {
                let concrete = value
                    .downcast_ref::<$ty>()
                    .expect("type mismatch in to_ron_component");
                ron::ser::to_string_pretty(concrete, ron::ser::PrettyConfig::default())
                    .expect("failed to serialize component")
            }
            fn __from_ron_component(text: String) -> Box<dyn std::any::Any> {
                let concrete: $ty =
                    ron::de::from_str(&text).expect("failed to deserialize component");
                Box::new(concrete) as Box<dyn std::any::Any>
            }
        }

        // Register the component
        inventory::submit! {
            $crate::ecs::component_registry::ComponentReg {
                type_name: <$ty>::TYPE_NAME,
                type_id: std::any::TypeId::of::<
                    $crate::ecs::component::ComponentStore<$ty>
                >(),
                to_ron: <$ty>::__to_ron,
                from_ron: <$ty>::__from_ron,
                factory: <$ty>::__factory,
                has: $crate::ecs::component_registry::has_component::<$ty>,
                remove: $crate::ecs::component_registry::erase_from_store::<$ty>,
                inserter: $crate::ecs::component_registry::generic_inserter::<$ty>,
                clone: |world: &$crate::ecs::world_ecs::WorldEcs,
                         entity: $crate::ecs::entity::Entity| {
                    let store_any = world
                        .stores
                        .get(&std::any::TypeId::of::<
                            $crate::ecs::component::ComponentStore<$ty>
                        >())
                        .expect("store missing despite has() == true");
                    let component = {
                        let store = store_any
                            .downcast_ref::<
                                $crate::ecs::component::ComponentStore<$ty>
                            >()
                            .expect("type mismatch in store");
                        store
                            .get(entity)
                            .expect("has() returned true but component missing")
                            .clone()
                    };
                    Box::new(component) as Box<dyn std::any::Any>
                },
                to_ron_component: <$ty>::__to_ron_component,
                from_ron_component: <$ty>::__from_ron_component,
                post_create: |any: &mut dyn std::any::Any| {
                    // Down‑cast the erased component to the concrete type
                    let comp = any
                        .downcast_mut::<$ty>()
                        .expect(concat!(
                            "post_create: Type mismatch.",
                            stringify!($ty)
                        ));
                    // Forward to the concreate function
                    $func(comp);
                },
            }
        }
    };
}

// Collect all registrations into a slice that lives for the whole program.
inventory::collect!(ComponentReg);

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredComponent {
    pub type_name: String,
    pub data: String,
}

/// Default implementation used when a component does not need any post‑create work.
pub fn post_create(
    _any: &mut dyn Any,
) {}