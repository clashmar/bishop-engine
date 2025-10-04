// engine_core/src/ecs/capture.rs

/// Generates a `capture_entity` implementation that works for any component
/// registered with `ecs_component!`.
#[macro_export]
macro_rules! impl_capture_entity {
    () => {
        use $crate::ecs::world_ecs::WorldEcs;
        use $crate::ecs::entity::Entity;
        use $crate::ecs::component::ComponentEntry;
        use $crate::ecs::component_registry::ComponentReg;

        /// Walks the component registry and extracts every component the entity owns
        pub fn capture_entity(
            world_ecs: &mut WorldEcs,
            entity: Entity,
        ) -> Vec<ComponentEntry> {
            let mut bag = Vec::new();

            // Iterate over all component registrations
            for reg in inventory::iter::<ComponentReg> {
                // Does this entity own the component?
                if (reg.has)(world_ecs, entity) {
                    // The registry now knows how to clone the concrete component
                    let any_val = (reg.clone)(world_ecs, entity);
                    // The inserter function is also stored in the registry entry
                    let inserter = reg.inserter;
                    bag.push(ComponentEntry {
                        value: any_val,
                        inserter,
                    });
                }
            }
            bag
        }
    };
}

impl_capture_entity!();