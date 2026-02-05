// engine_core/src/ecs/transform.rs
use crate::ecs::entity::*;
use crate::ecs::ecs::Ecs;
use crate::inspector_module;
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use macroquad::prelude::*;
use serde_with::serde_as;
use serde_with::FromInto;
use reflect_derive::Reflect;

/// Pivot point for sprite rendering. Defines which point on the sprite
/// aligns with the entity's Transform position.
#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Pivot {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    #[default]
    BottomCenter,
    BottomRight,
}

impl Pivot {
    /// Returns normalized offset (0.0-1.0) where (0,0)=top-left, (1,1)=bottom-right.
    pub fn as_normalized(&self) -> Vec2 {
        match self {
            Pivot::TopLeft => vec2(0.0, 0.0),
            Pivot::TopCenter => vec2(0.5, 0.0),
            Pivot::TopRight => vec2(1.0, 0.0),
            Pivot::CenterLeft => vec2(0.0, 0.5),
            Pivot::Center => vec2(0.5, 0.5),
            Pivot::CenterRight => vec2(1.0, 0.5),
            Pivot::BottomLeft => vec2(0.0, 1.0),
            Pivot::BottomCenter => vec2(0.5, 1.0),
            Pivot::BottomRight => vec2(1.0, 1.0),
        }
    }

    /// All variants for UI dropdowns.
    pub fn all() -> &'static [Pivot] {
        &[
            Pivot::TopLeft,
            Pivot::TopCenter,
            Pivot::TopRight,
            Pivot::CenterLeft,
            Pivot::Center,
            Pivot::CenterRight,
            Pivot::BottomLeft,
            Pivot::BottomCenter,
            Pivot::BottomRight,
        ]
    }

    /// Display label for UI.
    pub fn label(&self) -> &'static str {
        match self {
            Pivot::TopLeft => "Top Left",
            Pivot::TopCenter => "Top Center",
            Pivot::TopRight => "Top Right",
            Pivot::CenterLeft => "Center Left",
            Pivot::Center => "Center",
            Pivot::CenterRight => "Center Right",
            Pivot::BottomLeft => "Bottom Left",
            Pivot::BottomCenter => "Bottom Center",
            Pivot::BottomRight => "Bottom Right",
        }
    }
}

impl std::fmt::Display for Pivot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Calculates the top-left corner position for a rectangle 
/// given an entity position, the rectangle's size, and a pivot point.
#[inline]
pub fn pivot_offset(entity_pos: Vec2, size: Vec2, pivot: Pivot) -> Vec2 {
    let offset = pivot.as_normalized();
    vec2(
        entity_pos.x - size.x * offset.x,
        entity_pos.y - size.y * offset.y,
    )
}

/// Transform component for entities.
#[ecs_component]
#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, Reflect)]
#[serde(default)]
pub struct Transform {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
    /// Pivot point for rendering. Defaults to BottomCenter.
    pub pivot: Pivot,
}
inspector_module!(Transform, removable = false);

/// Update the position of an entity and any children it may have.
pub fn update_entity_position(ecs: &mut Ecs, entity: Entity, new_pos: Vec2) {
    // Determine the old position
    let old_pos = if let Some(pos) = ecs.get_store_mut::<Transform>().get_mut(entity) {
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
        let child_new_pos = if let Some(child_pos) = ecs.get_store_mut::<Transform>().get_mut(child) {
            let new = child_pos.position + delta;
            child_pos.position = new;
            new
        } else {
            return;
        };

        update_entity_position(ecs, child, child_new_pos);
    }
}
