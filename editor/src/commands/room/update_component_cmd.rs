// editor/src/commands/room/update_component_cmd.rs
use crate::commands::editor_command_manager::EditorCommand;
use crate::app::EditorMode;
use crate::with_editor;
use engine_core::prelude::*;

/// Undo-able command for editing a single component field via the inspector.
#[derive(Debug)]
pub struct UpdateComponentCmd {
    entity: Entity,
    room_id: RoomId,
    type_name: &'static str,
    old_ron: String,
    new_ron: String,
}

impl UpdateComponentCmd {
    pub fn new(
        entity: Entity,
        room_id: RoomId,
        type_name: &'static str,
        old_ron: String,
        new_ron: String,
    ) -> Self {
        Self { entity, room_id, type_name, old_ron, new_ron }
    }

    fn apply(entity: Entity, type_name: &'static str, ron: String, editor: &mut crate::Editor) {
        let ctx = &mut editor.game.ctx_mut();
        if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == type_name) {
            if (reg.has)(ctx.ecs, entity) {
                let mut old = (reg.clone)(ctx.ecs, entity);
                (reg.post_remove)(&mut *old, &entity, ctx);
            }
            let mut boxed = (reg.from_ron_component)(ron);
            (reg.post_create)(&mut *boxed, &entity, ctx);
            (reg.inserter)(ctx.ecs, entity, boxed);
        }
    }
}

impl EditorCommand for UpdateComponentCmd {
    fn execute(&mut self) {
        let type_name = self.type_name;
        let ron = self.new_ron.clone();
        let entity = self.entity;
        with_editor(|editor| Self::apply(entity, type_name, ron, editor));
    }

    fn undo(&mut self) {
        let type_name = self.type_name;
        let ron = self.old_ron.clone();
        let entity = self.entity;
        with_editor(|editor| Self::apply(entity, type_name, ron, editor));
    }

    fn mode(&self) -> EditorMode {
        EditorMode::Room(self.room_id)
    }
}
