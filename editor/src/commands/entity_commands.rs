// editor/src/commands/entity_commands.rs
use engine_core::ecs::{
    capture::capture_entity, 
    component::ComponentEntry, 
    entity::Entity, 
    world_ecs::WorldEcs
};
use crate::{
    commands::command_manager::Command, 
    global::with_editor
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
        });
    }

    fn undo(&mut self) {
        // Recreate the entity and put its components back together
        if let Some(bag) = self.saved.take() {
            with_editor(|editor| {
                let world_ecs = &mut editor.world.world_ecs;
                // Create a fresh entity
                let new_entity = world_ecs.create_entity().finish();
                self.entity = new_entity;
                restore_entity(world_ecs, new_entity, bag);
            });
        }
    }
}

fn restore_entity(world_ecs: &mut WorldEcs, entity: Entity, bag: Vec<ComponentEntry>) {
    for entry in bag {
        (entry.inserter)(world_ecs, entity, entry.value);
    }
}

