// editor/src/commands/entity_commands.rs
use engine_core::ecs::{
    capture::capture_entity, 
    component::ComponentEntry, 
    entity::Entity, 
    world_ecs::WorldEcs
};
use crate::{
    commands::command_manager::Command, 
    global::*
};

pub struct DeleteEntityCmd {
    pub entity: Entity,
    pub saved: Option<Vec<ComponentEntry>>,
}

impl Command for DeleteEntityCmd {
    fn execute(&mut self) {
        // Capture components before deleting
        with_editor(|editor| {
            let world_ecs = &mut editor.world.world_ecs;
            self.saved = Some(capture_entity(world_ecs, self.entity));
            world_ecs.remove_entity(self.entity); // delete
            editor.room_editor.set_selected_entity(None);
        });
    }

    fn undo(&mut self) {
        // Recreate the entity and put its components back together
        if let Some(bag) = self.saved.take() {
            with_editor(|editor| {
                let world_ecs = &mut editor.world.world_ecs;
                restore_entity(world_ecs, self.entity, bag);
            });
        }
    }
}

fn restore_entity(world_ecs: &mut WorldEcs, entity: Entity, bag: Vec<ComponentEntry>) {
    for entry in bag {
        (entry.inserter)(world_ecs, entity, entry.value);
    }
}

/// Copy a snapshot of the entity to the entity clipboard.
pub fn copy_entity(world_ecs: &mut WorldEcs, entity: Entity) {
    let snapshot = capture_entity(world_ecs, entity); 

    SERVICES.with(|s| {
        *s.entity_clipboard.borrow_mut() = Some(snapshot);
    });
}

/// Creates a new entity from the entity clipboard.
pub struct PasteEntityCmd {
    /// The entity that was created by the most recent paste.
    entity: Option<Entity>,
}

impl PasteEntityCmd {
    pub fn new() -> Self {
        Self { entity: None }
    }
}

impl Command for PasteEntityCmd {
    fn execute(&mut self) {
        let clipboard = SERVICES.with(|s| {
            s.entity_clipboard
                .borrow()
                .as_ref()
                .cloned()
        });

        if let Some(snapshot) = clipboard {
            with_editor(|editor| {
                let world_ecs = &mut editor.world.world_ecs;
                let new_entity = world_ecs.create_entity().finish();

                for component_entry in snapshot {
                    let entry = component_entry.clone(); // deep‑clone the boxed component
                    (entry.inserter)(world_ecs, new_entity, entry.value);
                }

                self.entity = Some(new_entity);
                editor.room_editor.set_selected_entity(Some(new_entity));
            });
        }
    }

    fn undo(&mut self) {
        if let Some(entity) = self.entity.take() {
            with_editor(|editor| {
                let world_ecs = &mut editor.world.world_ecs;
                world_ecs.remove_entity(entity); // delete
                editor.room_editor.set_selected_entity(None);
            });
        }
    }
}