// engine_core/src/world/transition_manager.rs
use crate::game_state::GameState;
use engine_core::engine_global::tile_size;
use engine_core::ecs::component::*;
use engine_core::world::room::*;
use macroquad::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionState {
    /// Normal state.
    #[default]
    None,
    /// Player has just crossed an exit boundary and still overlaps both rooms.
    Penetrated,
    /// Player is completely inside the target room.
    Entered,
    /// Player moved back into the previous room from overlapping state.
    Retreated,
}

pub struct TransitionManager {
    pub state: TransitionState,
    pub from: Option<Uuid>,
    pub to: Option<Uuid>,
}

impl TransitionManager {
    pub fn new() -> Self {
        Self {
            state: TransitionState::None,
            from: None,
            to: None,
        }
    }

    /// Called when the physics system reports that the player crossed an exit.
    pub fn set_state(&mut self, new_state: TransitionState, target_room: Uuid) {
        match new_state {
            TransitionState::Penetrated => {
                self.from = self.to;
                self.to = Some(target_room);
            }
            TransitionState::Entered => {
                self.state = TransitionState::None;
            }
            TransitionState::Retreated => {
                self.from = Some(target_room);
                self.to = None;
            }
            TransitionState::None => {}
        }
        self.state = new_state;
    }

    /// Helper to query if currently in a transition.
    pub fn in_transition(&self) -> bool {
        matches!(self.state, TransitionState::Penetrated | TransitionState::Retreated)
    }

    /// Handles entity transitions between rooms.
    pub fn handle_transitions(
        game_state: &mut GameState,
    ) {
        let world = game_state.game.current_world_mut();
        let rooms = world.rooms.clone();
        let world_ecs = &mut world.world_ecs;
        
        let entities: Vec<_> = world_ecs
            .get_store::<Position>()
            .data
            .keys()
            .cloned()
            .collect();

        for entity in entities {
            let (pos, _coll) = {
                let p = match world_ecs.get::<Position>(entity) {
                    Some(v) => v.position,
                    None => continue,           
                };
                let c = match world_ecs.get::<Collider>(entity) {
                    Some(v) => v,
                    None => continue,           
                };
                (p, c)
            };

            // Find the room that now contains the entity
            let target_id = match room_of_entity(pos, &rooms) {
                Some(id) => id,
                None => return,
            }; 

            if let Some(comp) = world_ecs.get_mut::<CurrentRoom>(entity) {
                if comp.0 == target_id {
                    return;
                } else {
                    comp.0 = target_id
                }
            }

            if world_ecs.get_player_entity() == entity {
                if let Some(new_room) = rooms.iter().find(|r| r.id == target_id) {
                    world.current_room_id = Some(new_room.id);
                }
            }
        }
    }
}

/// Return the id of the room whose bounds contain the entity’s AABB.
pub fn room_of_entity(pos: Vec2, rooms: &[Room]) -> Option<RoomId> {
    // TODO: work out position based on collider after we've worked out drawing
    for room in rooms {
        let min = room.position;
        let max = room.position + room.size * tile_size();

        if pos.x >= min.x
            && pos.x <= max.x
            && pos.y >= min.y
            && pos.y <= max.y
        {
            return Some(room.id);
        }
    }
    None
}

