// editor/src/commands/entity_commands.rs
use crate::commands::editor_command_manager::EditorCommand;
use crate::ecs::component_registry::ComponentRegistry;
use crate::editor::EditorMode;
use crate::ecs::position::*;
use crate::EDITOR_SERVICES;
use crate::ecs::entity::*;
use crate::ecs::ecs::Ecs;
use crate::with_editor;
use engine_core::ecs::component::comp_type_name;
use engine_core::world::room::RoomId;
use engine_core::ecs::capture::*;
use std::collections::HashMap;
use macroquad::prelude::*;

#[derive(Debug)]
pub struct DeleteEntityCmd {
    pub entity: Entity,
    pub saved: Option<Vec<(Entity, Vec<(String, String)>)>>,
}

impl EditorCommand for DeleteEntityCmd {
    fn execute(&mut self) {
        // Capture components before deleting
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            self.saved = Some(capture_subtree(ecs, self.entity));
            ecs.remove_entity(self.entity);
            editor.room_editor.set_selected_entity(None);
        });
    }

    fn undo(&mut self) {
        if let Some(saved) = self.saved.take() {
            with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                // Restore every entity and its components
                restore_subtree(ecs, &saved);
                editor.room_editor.set_selected_entity(Some(self.entity));
            });
        }
    }

    fn mode(&self) -> EditorMode { 
        EditorMode::Room(RoomId::default())
    }
}

/// Copy a snapshot of the entity and its children to the global clipboard.
pub fn copy_entity(ecs: &mut Ecs, entity: Entity) {
    let snapshot = capture_subtree(ecs, entity);
    EDITOR_SERVICES.with(|s| {
        *s.entity_clipboard.borrow_mut() = Some(snapshot);
    });
}

/// Creates a new entity from the entity clipboard.
#[derive(Debug)]
pub struct PasteEntityCmd {
    /// The entity that was created by the most recent paste.
    id_map: Option<HashMap<Entity, Entity>>,
    /// The component snapshot that was taken the first time the command ran.
    snapshot: Option<Vec<(Entity, Vec<(String, String)>)>>,
}

impl PasteEntityCmd {
    pub fn new() -> Self {
        Self { 
            id_map: None,
            snapshot: None,
         }
    }
}

impl EditorCommand for PasteEntityCmd {
    fn execute(&mut self) {
        if self.snapshot.is_none() {
            self.snapshot = EDITOR_SERVICES.with(|s| s.entity_clipboard.borrow().clone());
        }
        let snapshot = match &self.snapshot {
            Some(s) => s,
            None => return,
        };

        let mut map = HashMap::new();
        for (old_id, _) in snapshot.iter() {
            let new_id = with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                ecs.create_entity().finish()
            });
            map.insert(*old_id, new_id);
        }
        self.id_map = Some(map.clone());

        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;

            for (old_id, bag) in snapshot.iter() {
                let new_id = map[old_id];

                for (type_name, ron) in bag.iter() {
                    // Look up the registry entry for this component type
                    let component_reg = inventory::iter::<ComponentRegistry>()
                        .find(|r| r.type_name == type_name)
                        .expect("Component not registered");

                    // Deserialize a fresh boxed component
                    let mut boxed = (component_reg.from_ron_component)(ron.clone());

                    if type_name == comp_type_name::<Parent>() {
                        let parent = boxed
                            .as_mut()
                            .downcast_mut::<Parent>()
                            .expect("Parent component type mismatch");

                        // Replace the old parent id with the newly created one
                        if let Some(&new_parent) = map.get(&parent.0) {
                            parent.0 = new_parent;
                        }
                    } else if type_name == comp_type_name::<Children>() {
                        let children = boxed
                            .as_mut()
                            .downcast_mut::<Children>()
                            .expect("Children component type mismatch");

                        // Translate every child id
                        for child in &mut children.entities {
                            if let Some(&new_child) = map.get(child) {
                                *child = new_child;
                            }
                        }
                    }

                    // Run any post‑create logic the component may have
                    (component_reg.post_create)(&mut *boxed);
                    // Insert it into the world under the new id
                    (component_reg.inserter)(ecs, new_id, boxed);
                }
            }

            let root_old = snapshot[0].0;
            let root_new = map[&root_old];
            editor.room_editor.set_selected_entity(Some(root_new));
        });
    }

    fn undo(&mut self) {
        if let Some(map) = &self.id_map {
            if let Some((root_old, _)) = self.snapshot.as_ref().and_then(|s| s.first()) {
                if let Some(&root_new) = map.get(root_old) {
                    with_editor(|editor| {
                        let ecs = &mut editor.game.ecs;
                        ecs.remove_entity(root_new);
                        editor.room_editor.set_selected_entity(None);
                    });
                }
            }
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
}

impl EditorCommand for MoveEntityCmd {
    fn execute(&mut self) {
        // Called the first time
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            update_entity_position(ecs, self.entity, self.to);
        });
        self.executed = true;
    }

    fn undo(&mut self) {
        // Restore the old position
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            update_entity_position(ecs, self.entity, self.from);
        });
        self.executed = false;
    }

    fn mode(&self) -> EditorMode { 
        EditorMode::Room(RoomId::default())
    }
}