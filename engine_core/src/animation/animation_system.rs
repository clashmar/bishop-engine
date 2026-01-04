// engine_core/src/animation/animation_system.rs
use crate::assets::asset_manager::AssetManager;
use crate::world::room::entities_in_room;
use crate::animation::animation_clip::*;
use crate::assets::sprite::SpriteId;
use crate::ecs::entity::Entity;
use crate::world::room::RoomId;
use crate::ecs::ecs::Ecs;
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use macroquad::prelude::*;

#[ecs_component]
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct CurrentFrame {
    #[serde(skip)]
    pub clip_id: ClipId,
    #[serde(skip)]
    pub col: usize,
    #[serde(skip)]
    pub row: usize,
    #[serde(skip)]
    pub offset: Vec2,
    #[serde(skip)]
    pub sprite_id: SpriteId,
    #[serde(skip)]
    pub frame_size: Vec2,
}

pub async fn update_animation_sytem(
    ecs: &mut Ecs,
    asset_manager: &mut AssetManager,
    dt: f32,
    room_id: RoomId,
) {
    // Gather the ids of all entities that are in the current room
    let entities = entities_in_room(ecs, room_id);

    let anim_store = ecs.get_store_mut::<Animation>();

    let mut frames: Vec<(Entity, CurrentFrame)> = vec![];

    for (entity, animation) in anim_store.data.iter_mut() {
        if !entities.contains(entity) {
            continue;
        }

        // Bail out early if there is no active clip.
        let Some(current_id) = &animation.current.clone() else { continue };

        // Get the sprite id
        let (sprite_id, resolved) = get_sprite_id(animation, current_id, asset_manager).await;

        if resolved {
            animation.update_cache_entry(current_id, sprite_id);
        }

        let Some(clip) = animation.clips.get(current_id) else { continue };
        let clip_state = animation.states.get_mut(current_id).unwrap();

        // Advance the timer
        clip_state.timer += dt;
        let frame_time = 1.0 / clip.fps.max(0.001);
        while clip_state.timer >= frame_time {
            clip_state.timer -= frame_time;
            clip_state.col += 1;
            if clip_state.col >= clip.cols {
                clip_state.col = 0;
                clip_state.row += 1;
                if clip_state.row >= clip.rows {
                    if clip.looping {
                        clip_state.row = 0;
                    } else {
                        clip_state.finished = true;
                        clip_state.row = clip.rows - 1;
                        clip_state.col = clip.cols - 1;
                    }
                }
            }
        }


        let frame = CurrentFrame {
            clip_id: animation.current.clone().unwrap(),
            col: clip_state.col,
            row: clip_state.row,
            offset: clip.offset,
            sprite_id: sprite_id,
            frame_size: clip.frame_size,
        };

        frames.push((*entity, frame));
    }

    

    for (entity, frame) in frames {
        ecs.add_component_to_entity(entity, frame)
    }
}

/// Return the SpriteId for for the current animation clip.
async fn get_sprite_id(
    animation: &Animation,
    current_id: &ClipId,
    asset_manager: &mut AssetManager,
) -> (SpriteId, bool) {
    // Try cache first
    if let Some(&cached) = animation.sprite_cache.get(current_id) {
        if cached.0 != 0 {
            return (cached, false);
        }
    }

    // Not in cache try to resolve with asset manager
    let resolved = resolve_sprite_id(
        asset_manager, 
        &animation.variant, 
        current_id
    ).await;

    (resolved, true)
}