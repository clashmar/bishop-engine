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
                // Does the entity own the component
                if (reg.has)(world_ecs, entity) {
                    let any_val = (reg.clone)(world_ecs, entity);
                    let inserter = reg.inserter;
                    let cloner = reg.clone_box;
                    bag.push(ComponentEntry {
                        value: any_val,
                        inserter,
                        cloner,
                    });
                }
            }
            bag
        }
    };
}

impl_capture_entity!();