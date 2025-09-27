// engine_core/src/animation/animation_system.rs
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    animation::animation_clip::{
        Animation, 
        ClipId
    }, assets::sprite::SpriteId, ecs::{
        entity::Entity, 
        world_ecs::WorldEcs
    }, ecs_component
};

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

ecs_component!(CurrentFrame);

pub fn update_animation_sytem(world_ecs: &mut WorldEcs, delta_time: f32) {
    let anim_store = world_ecs.get_store_mut::<Animation>();

    let mut frames: Vec<(Entity, CurrentFrame)> = vec![];

    for (entity, animation) in anim_store.data.iter_mut() {
        // Bail out early if there is no active clip.
        let Some(current_id) = &animation.current else { continue };
        let Some(clip) = animation.clips.get(current_id) else { continue };
        let clip_state = animation.states.get_mut(current_id).unwrap();

        // Advance the timer
        clip_state.timer += delta_time;
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
            sprite_id: clip.sprite_id,
            frame_size: clip.frame_size,
        };

        frames.push((*entity, frame));
    }

    for (entity, frame) in frames {
        world_ecs.add_component_to_entity(entity, frame)
    }
}