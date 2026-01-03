// engine_core/src/ecs/capture.rs

/// Generates a `capture_entity` implementation that works for any component
/// registered with `ecs_component!`.
#[macro_export]
macro_rules! impl_capture_entity {
    () => {
        use $crate::ecs::ecs::Ecs;
        use $crate::ecs::entity::Entity;
        use $crate::ecs::component_registry::ComponentRegistry;

        /// Walks the component registry and extracts every component the entity owns
        pub fn capture_entity(
            ecs: &mut Ecs,
            entity: Entity,
        ) -> Vec<(String, String)>{
            let mut bag = Vec::new();

            // Iterate over all component registrations
            for component_reg in inventory::iter::<ComponentRegistry> {
                // Does the entity own the component
                if (component_reg.has)(ecs, entity) {
                    // Serialize the *component* (not the whole store) to a RON string.
                    // `reg.clone` gives us a boxed component value.
                    let boxed = (component_reg.clone)(ecs, entity);
                    let ron = (component_reg.to_ron_component)(&*boxed);
                    bag.push((component_reg.type_name.to_string(), ron));
                }
            }
            bag
        }
    };
}

impl_capture_entity!();