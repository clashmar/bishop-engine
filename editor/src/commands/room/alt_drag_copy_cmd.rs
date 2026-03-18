// editor/src/commands/room/alt_drag_copy_cmd.rs
use crate::commands::editor_command_manager::EditorCommand;
use crate::ecs::component_registry::ComponentRegistry;
use crate::app::EditorMode;
use crate::ecs::entity::*;
use crate::ecs::ecs::Ecs;
use crate::with_editor;
use engine_core::ecs::component::comp_type_name;
use engine_core::ecs::capture::capture_subtree;
use engine_core::world::room::RoomId;
use std::collections::{HashMap, HashSet};

/// Undo-able command for alt+drag copy operation.
/// Unlike DuplicateEntitiesCmd, this command is created after entities already exist
/// (they were created during the drag operation).
#[derive(Debug)]
pub struct AltDragCopyCmd {
    room_id: RoomId,
    /// The entities that were created during alt+drag.
    created_entities: Vec<Entity>,
    /// Snapshot captured from created entities for redo.
    snapshot: Option<Vec<(Entity, Vec<(String, String)>)>>,
    /// Whether this is the first execution (entities already exist).
    first_execute: bool,
    /// Maps old entity IDs to newly created ones (for redo).
    id_map: Option<HashMap<Entity, Entity>>,
}

impl AltDragCopyCmd {
    /// Create a new AltDragCopyCmd with the entities that were created during drag.
    pub fn new(created_entities: Vec<Entity>, room_id: RoomId) -> Self {
        Self {
            room_id,
            created_entities,
            snapshot: None,
            first_execute: true,
            id_map: None,
        }
    }
}

impl EditorCommand for AltDragCopyCmd {
    fn execute(&mut self) {
        if self.first_execute {
            // First execute: entities already exist from drag operation.
            // Capture their snapshot for potential redo.
            let mut all_snapshots = Vec::new();
            with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                for &entity in &self.created_entities {
                    let snapshot = capture_subtree(ecs, entity);
                    all_snapshots.extend(snapshot);
                }
            });
            self.snapshot = Some(all_snapshots.clone());
            self.first_execute = false;

            // Call post_create for all components
            with_editor(|editor| {
                let ctx = &mut editor.game.ctx_mut();

                for (entity_id, bag) in &all_snapshots {
                    for (type_name, _) in bag {
                        let component_reg = match inventory::iter::<ComponentRegistry>()
                            .find(|r| r.type_name == type_name)
                        {
                            Some(reg) => reg,
                            None => continue,
                        };

                        // Clone the existing component, call post_create, then reinsert
                        let mut boxed = (component_reg.clone)(ctx.ecs, *entity_id);
                        (component_reg.post_create)(&mut *boxed, entity_id, ctx);
                        (component_reg.inserter)(ctx.ecs, *entity_id, boxed);
                    }
                }
            });

            // Select the created entities
            with_editor(|editor| {
                editor.room_editor.clear_selection();
                for &entity in &self.created_entities {
                    editor.room_editor.add_to_selection(entity);
                }
            });
            return;
        }

        // Redo: recreate entities from snapshot
        let snapshot = match &self.snapshot {
            Some(s) if !s.is_empty() => s,
            _ => return,
        };

        // Find root entities (no parent in snapshot)
        let snapshot_ids: HashSet<Entity> = snapshot.iter().map(|(id, _)| *id).collect();
        let mut root_old_ids = Vec::new();

        for (old_id, bag) in snapshot.iter() {
            let has_parent_in_snapshot = bag.iter().any(|(type_name, ron)| {
                if type_name == comp_type_name::<Parent>() {
                    if let Ok(parent) = ron::from_str::<Parent>(ron) {
                        return snapshot_ids.contains(&parent.0);
                    }
                }
                false
            });

            if !has_parent_in_snapshot {
                root_old_ids.push(*old_id);
            }
        }

        // Create new entities
        let mut map = HashMap::new();
        for (old_id, _) in snapshot.iter() {
            let new_id = with_editor(|editor| {
                let ecs = &mut editor.game.ecs;
                ecs.create_entity().finish()
            });
            map.insert(*old_id, new_id);
        }
        self.id_map = Some(map.clone());
        self.created_entities = root_old_ids.iter().filter_map(|old| map.get(old).copied()).collect();

        with_editor(|editor| {
            let ctx = &mut editor.game.ctx_mut();

            for (old_id, bag) in snapshot.iter() {
                let new_id = map[old_id];

                for (type_name, ron) in bag.iter() {
                    let component_reg = inventory::iter::<ComponentRegistry>()
                        .find(|r| r.type_name == type_name)
                        .expect("Component not registered");

                    let mut boxed = (component_reg.from_ron_component)(ron.clone());

                    if type_name == comp_type_name::<Parent>() {
                        let parent = boxed
                            .as_mut()
                            .downcast_mut::<Parent>()
                            .expect("Parent component type mismatch");

                        if let Some(&new_parent) = map.get(&parent.0) {
                            parent.0 = new_parent;
                        }
                    } else if type_name == comp_type_name::<Children>() {
                        let children = boxed
                            .as_mut()
                            .downcast_mut::<Children>()
                            .expect("Children component type mismatch");

                        for child in &mut children.entities {
                            if let Some(&new_child) = map.get(child) {
                                *child = new_child;
                            }
                        }
                    }

                    (component_reg.post_create)(&mut *boxed, &new_id, ctx);
                    (component_reg.inserter)(ctx.ecs, new_id, boxed);
                }
            }

            // Select all recreated root entities
            editor.room_editor.clear_selection();
            for &root in &self.created_entities {
                editor.room_editor.add_to_selection(root);
            }
        });
    }

    fn undo(&mut self) {
        with_editor(|editor| {
            let ctx = &mut editor.game.ctx_mut();

            for &entity in &self.created_entities {
                Ecs::remove_entity(ctx, entity);
            }

            editor.room_editor.clear_selection();
        });
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Room(self.room_id)
    }
}
