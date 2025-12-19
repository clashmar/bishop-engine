// engine_core/src/scripting/interactable.rs
use crate::{ecs::component::CurrentRoom, inspector_module};
use crate::ecs::world_ecs::WorldEcs;
use crate::ecs::component::Position;
use crate::ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use reflect_derive::Reflect;

/// Component for interactable entities.
#[ecs_component]
#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect)]
pub struct Interactable {
    /// Maximum interaction distance.
    pub range: f32,
    // TODO: Add priority,
    // enabled/disabled,
    // prompt,
    // facing
    // event dispatch
}
inspector_module!(Interactable);

/// Returns the best interactable entity candidate for the player in the `CurrentRoom` or `None`.
pub fn find_best_interactable(world_ecs: &WorldEcs) -> Option<Entity> {
    let player = world_ecs.get_player_entity();
    let player_pos = world_ecs.get_player_position().position;

    let player_room = world_ecs
        .get::<CurrentRoom>(player)
        .map(|r| r.0)?;

    let interactables = world_ecs.get_store::<Interactable>();
    let positions = world_ecs.get_store::<Position>();
    let rooms = world_ecs.get_store::<CurrentRoom>();

    let mut best: Option<(Entity, f32)> = None;

    for (entity, interactable) in &interactables.data {
        // Must have position and room
        let pos = match positions.get(*entity) {
            Some(p) => p.position,
            None => continue,
        };

        let room = match rooms.get(*entity) {
            Some(r) => r.0,
            None => continue,
        };

        if room != player_room {
            continue;
        }

        // TODO: rethink how position is set for this purpose
        let dist = player_pos.distance(pos);

        if dist > interactable.range {
            continue;
        }

        match best {
            None => best = Some((*entity, dist)),
            Some((_, best_dist)) if dist < best_dist => {
                best = Some((*entity, dist))
            }
            _ => {}
        }
    }

    best.map(|(entity, _)| entity)
}
