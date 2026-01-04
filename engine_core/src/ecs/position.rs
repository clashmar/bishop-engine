// engine_core/src/ecs/position.rs
use crate::ecs::entity::*;
use crate::ecs::ecs::Ecs;
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use macroquad::prelude::*;
use serde_with::serde_as;
use serde_with::FromInto;

/// World position of the entity.
#[ecs_component]
#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Position {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
}

/// Update the position of an entity and any children it may have.
pub fn update_entity_position(ecs: &mut Ecs, entity: Entity, new_pos: Vec2) {
    // Determine the old position 
    let old_pos = if let Some(pos) = ecs.get_store_mut::<Position>().get_mut(entity) {
        let old = pos.position;
        pos.position = new_pos;
        old
    } else {
        return;
    };

    // Compute the translation that has to be applied to the children
    let delta = new_pos - old_pos;
    if delta == Vec2::ZERO {
        return;
    }

    // Propagate the translation to every child recursively
    let children = get_children(ecs, entity);
    for child in children {
        let child_new_pos = if let Some(child_pos) = ecs.get_store_mut::<Position>().get_mut(child) {
            let new = child_pos.position + delta;
            child_pos.position = new;
            new
        } else {
            return;
        };

        update_entity_position(ecs, child, child_new_pos);
    }
}