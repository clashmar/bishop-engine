// editor/src/commands/room/move_entity_cmd.rs
use crate::commands::editor_command_manager::EditorCommand;
use crate::editor::EditorMode;
use crate::with_editor;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::transform::update_entity_position;
use engine_core::world::room::RoomId;
use macroquad::prelude::*;

/// Undo-able command for moving an entity.
#[derive(Debug)]
pub struct MoveEntityCmd {
    entity: Entity,
    room_id: RoomId,
    from: Vec2,
    to: Vec2,
    executed: bool,
}

impl MoveEntityCmd {
    pub fn new(entity: Entity, room_id: RoomId, from: Vec2, to: Vec2) -> Self {
        Self {
            entity,
            room_id,
            from,
            to,
            executed: false,
        }
    }
}

impl EditorCommand for MoveEntityCmd {
    fn execute(&mut self) {
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            update_entity_position(ecs, self.entity, self.to);
        });
        self.executed = true;
    }

    fn undo(&mut self) {
        with_editor(|editor| {
            let ecs = &mut editor.game.ecs;
            update_entity_position(ecs, self.entity, self.from);
        });
        self.executed = false;
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Room(self.room_id)
    }
}
