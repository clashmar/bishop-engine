// editor/src/commands/room/copy_entity.rs
use crate::EDITOR_SERVICES;
use crate::ecs::ecs::Ecs;
use engine_core::ecs::component::Player;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::capture::capture_subtree;

/// Copy a snapshot of the entity and its children to the global clipboard.
/// Returns false if the entity is a Player (copying Player is not allowed).
pub fn copy_entity(ecs: &mut Ecs, entity: Entity) -> bool {
    // Block copying Player entities
    if ecs.has::<Player>(entity) {
        return false;
    }

    let snapshot = capture_subtree(ecs, entity);
    EDITOR_SERVICES.with(|s| {
        *s.entity_clipboard.borrow_mut() = Some(snapshot);
    });
    true
}
