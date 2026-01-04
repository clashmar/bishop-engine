// engine_core/src/ecs/capture.rs
use crate::ecs::entity::Children;

/// Capture the whole entity hierarchy that starts at `root`.
/// Returns a vector of (old_entity, component_bag) for the root and every descendant.
pub fn capture_subtree(ecs: &mut Ecs, root: Entity) -> Vec<(Entity, Vec<(String, String)>)> {
    let mut result = Vec::new();
    let mut stack = vec![root];

    while let Some(e) = stack.pop() {
        // Capture the components of the current entity
        let bag = capture_entity(ecs, e);
        result.push((e, bag));

        // Push its children (if any) on the stack
        if let Some(children) = ecs.get::<Children>(e) {
            for &c in &children.entities {
                stack.push(c);
            }
        }
    }
    result
}

/// Restore a previously captured subtree.  
pub fn restore_subtree(ecs: &mut Ecs, saved: &[(Entity, Vec<(String, String)>)]) {
    // Create every entity id that appears in the snapshot
    for (old_id, _) in saved {
        let _ = *old_id;
    }

    // Insert the component bags
    for (old_id, bag) in saved {
        restore_entity(ecs, *old_id, bag.clone());
    }
}

/// Restores an entity into the Ecs from its component bag.
pub fn restore_entity(
    ecs: &mut Ecs,
    entity: Entity,
    bag: Vec<(String, String)>,
) {
    for (type_name, ron) in bag {
        // Look up the registry entry for this component type
        let component_reg = inventory::iter::<ComponentRegistry>()
            .find(|r| r.type_name == type_name)
            .expect("Component not registered");

        // Deserialize a fresh boxed component.
        let mut boxed = (component_reg.from_ron_component)(ron);

        // Run any post create logic the component may have
        (component_reg.post_create)(&mut *boxed);

        // Insert it into the (already‑existing) entity
        (component_reg.inserter)(ecs, entity, boxed);
    }
}

/// Generates a `capture_entity` implementation that works for any component
/// registered with `ecs_component!`.
#[macro_export]
macro_rules! impl_capture_entity {
    () => {
        use $crate::ecs::ecs::Ecs;
        use $crate::ecs::entity::Entity;
        use $crate::ecs::component_registry::ComponentRegistry;

        /// Walks the component registry and extracts every component the entity owns.
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