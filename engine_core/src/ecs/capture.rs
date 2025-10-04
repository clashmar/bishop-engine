// engine_core/src/ecs/capture.rs

/// Generates a `capture_entity` implementation that works for any component
/// registered with `ecs_component!`.
#[macro_export]
macro_rules! impl_capture_entity {
    () => {
        use $crate::ecs::world_ecs::WorldEcs;
        use $crate::ecs::entity::Entity;
        use $crate::ecs::component_registry::ComponentReg;

        /// Walks the component registry and extracts every component the entity owns
        pub fn capture_entity(
            world_ecs: &mut WorldEcs,
            entity: Entity,
        ) -> Vec<(String, String)>{
            let mut bag = Vec::new();

            // Iterate over all component registrations
            for reg in inventory::iter::<ComponentReg> {
                // Does the entity own the component
                if (reg.has)(world_ecs, entity) {
                    // Serialize the *component* (not the whole store) to a RON string.
                    // `reg.clone` gives us a boxed component value.
                    let boxed = (reg.clone)(world_ecs, entity);
                    let ron = (reg.to_ron_component)(&*boxed);
                    bag.push((reg.type_name.to_string(), ron));
                }
            }
            bag
        }
    };
}

impl_capture_entity!();