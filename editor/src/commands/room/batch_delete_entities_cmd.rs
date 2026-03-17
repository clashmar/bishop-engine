// editor/src/commands/room/batch_delete_entities_cmd.rs
use crate::commands::editor_command_manager::EditorCommand;
use crate::app::EditorMode;
use crate::with_editor;
use crate::ecs::ecs::Ecs;
use engine_core::ecs::entity::Entity;
use engine_core::world::room::RoomId;
use engine_core::ecs::capture::*;

/// Undo-able command for deleting multiple entities and their children.
#[derive(Debug)]
pub struct BatchDeleteEntitiesCmd {
    pub entities: Vec<Entity>,
    pub room_id: RoomId,
    pub saved: Option<Vec<(Entity, Vec<(String, String)>)>>,
}

impl BatchDeleteEntitiesCmd {
    pub fn new(entities: Vec<Entity>, room_id: RoomId) -> Self {
        Self {
            entities,
            room_id,
            saved: None,
        }
    }
}

impl EditorCommand for BatchDeleteEntitiesCmd {
    fn execute(&mut self) {
        with_editor(|editor| {
            let ctx = &mut editor.game.ctx_mut();

            // Capture all entities before deleting
            let mut all_saved = Vec::new();
            for &entity in &self.entities {
                let captured = capture_subtree(ctx.ecs, entity);
                all_saved.extend(captured);
            }
            self.saved = Some(all_saved);

            // Delete all entities
            for &entity in &self.entities {
                Ecs::remove_entity(ctx, entity);
            }

            editor.room_editor.clear_selection();
        });
    }

    fn undo(&mut self) {
        if let Some(saved) = self.saved.take() {
            with_editor(|editor| {
                let ctx = &mut editor.game.ctx_mut();
                restore_subtree(ctx, &saved);

                // Re-select the entities
                for &entity in &self.entities {
                    editor.room_editor.add_to_selection(entity);
                }
            });
        }
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Room(self.room_id)
    }
}
