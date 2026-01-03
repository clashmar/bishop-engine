// editor/src/commands/entity_commands.rs
use crate::commands::editor_command_manager::EditorCommand;
use crate::ecs::component_registry::ComponentRegistry;
use crate::ecs::capture::capture_entity;
use crate::ecs::component::Position;
use crate::ecs::entity::Entity;
use crate::editor::EditorMode;
use crate::EDITOR_SERVICES;
use crate::ecs::ecs::Ecs;
use crate::with_editor;
use engine_core::world::room::RoomId;
use macroquad::prelude::*;

#[derive(Debug)]
pub struct DeleteEntityCmd {
    pub entity: Entity,
    pub saved: Option<Vec<(String, String)>>,
}

impl EditorCommand for DeleteEntityCmd {
    fn execute(&mut self) {
        // Capture components before deleting
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            self.saved = Some(capture_entity(ecs, self.entity));
            ecs.remove_entity(self.entity); // delete
            editor.room_editor.set_selected_entity(None);
        });
    }

    fn undo(&mut self) {
        // Recreate the entity and put its components back together
        if let Some(bag) = self.saved.take() {
            with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                restore_entity(ecs, self.entity, bag);
            });
        }
    }

    fn mode(&self) -> EditorMode { 
        EditorMode::Room(RoomId::default())
    }
}

fn restore_entity(
    world_ecs: &mut Ecs,
    entity: Entity,
    bag: Vec<(String, String)>,
) {
    for (type_name, ron) in bag {
        // Look up the registry entry for this component type.
        let component_reg = inventory::iter::<ComponentRegistry>()
            .find(|r| r.type_name == type_name)
            .expect("Component not registered");

        // Deserialize a fresh boxed component.
        let mut boxed = (component_reg.from_ron_component)(ron);

        // Run any post create logic the component may have
        (component_reg.post_create)(&mut *boxed);

        // Insert it into the (already‑existing) entity.
        (component_reg.inserter)(world_ecs, entity, boxed);
    }
}

/// Copy a snapshot of the entity to the global entity clipboard.
pub fn copy_entity(world_ecs: &mut Ecs, entity: Entity) {
    let snapshot = capture_entity(world_ecs, entity);
    EDITOR_SERVICES.with(|s| {
        *s.entity_clipboard.borrow_mut() = Some(snapshot);
    });
}

/// Creates a new entity from the entity clipboard.
#[derive(Debug)]
pub struct PasteEntityCmd {
    /// The entity that was created by the most recent paste.
    entity: Option<Entity>,
    /// The component snapshot that was taken the first time the command ran.
    snapshot: Option<Vec<(String, String)>>,
}

impl PasteEntityCmd {
    pub fn new() -> Self {
        Self { 
            entity: None,
            snapshot: None,
         }
    }
}

impl EditorCommand for PasteEntityCmd {
    fn execute(&mut self) {
        // Grab the clipboard only once on first execution
        if self.snapshot.is_none() {
            self.snapshot = EDITOR_SERVICES.with(|s| s.entity_clipboard.borrow().clone());
        }

        // Bail out if nothing is on the clipboard
        let snapshot = match &self.snapshot {
            Some(s) => s,
            None => return,
        };

        // Ensure we have an Entity id
        if self.entity.is_none() {
            // Allocate a fresh UUID for the first execution
            self.entity = with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                Some(ecs.create_entity().finish())
            });
        }

        let entity = self.entity.expect("Entity must be set.");

        // Populate the component stores for that id
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            for (type_name, ron) in snapshot {
                // Find the registry entry for this component type
                let component_reg = inventory::iter::<ComponentRegistry>()
                    .find(|r| r.type_name == type_name)
                    .expect("Component not registered");

                // Deserialize a fresh boxed component
                let mut boxed = (component_reg.from_ron_component)(ron.clone());

                // Run any post‑create logic the component may have
                (component_reg.post_create)(&mut *boxed);

                // Insert it into the world under the same id
                (component_reg.inserter)(ecs, entity, boxed);
            }

            // Select the entity in the ui
            editor.room_editor.set_selected_entity(Some(entity));
        });
    }

    fn undo(&mut self) {
        // Remove the entity but keep the id for a later redo
        if let Some(entity) = self.entity {
            with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                ecs.remove_entity(entity);
                editor.room_editor.set_selected_entity(None);
            });
        }
    }

    fn mode(&self) -> EditorMode { 
        EditorMode::Room(RoomId::default())
    }
}

/// Undo-able move‑entity command.
#[derive(Debug)]
pub struct MoveEntityCmd {
    entity: Entity,
    from: Vec2,
    to: Vec2,
    executed: bool,
}

impl MoveEntityCmd {
    pub fn new(entity: Entity, from: Vec2, to: Vec2) -> Self {
        Self {
            entity,
            from,
            to,
            executed: false,
        }
    }

    /// Helper that writes a concrete position into the world.
    fn set_position(world_ecs: &mut Ecs, entity: Entity, position: Vec2) {
        if let Some(pos) = world_ecs
            .get_store_mut::<Position>()
            .get_mut(entity)
        {
            pos.position = position;
        }
    }
}

impl EditorCommand for MoveEntityCmd {
    fn execute(&mut self) {
        // Called the first time
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            Self::set_position(ecs, self.entity, self.to);
        });
        self.executed = true;
    }

    fn undo(&mut self) {
        // Restore the old position
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            Self::set_position(ecs, self.entity, self.from);
        });
        self.executed = false;
    }

    fn mode(&self) -> EditorMode { 
        EditorMode::Room(RoomId::default())
    }
}