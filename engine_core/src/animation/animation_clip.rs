// engine_core/src/animation/animation_clip.rs
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_with::{FromInto, serde_as};

use crate::{assets::sprite::SpriteId, ecs_component};

/// Logical name of a clip.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClipId {
    #[default]
    Idle,
    Walk,
    Run,
    Attack,
    Custom(String),
}

/// One animation clip inside a sprite sheet.
#[serde_as]
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Clip {
    /// Texture that holds the sheet.
    pub sprite_id: SpriteId,
    /// Width and height of a single cell.
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub frame_size: Vec2,
    /// Frames per row.
    pub cols: usize,
    /// Number of rows that belong to this clip.
    pub rows: usize,
    /// Playback speed in frames per second.
    pub fps: f32,
    /// Whether the clip loops.
    pub looping: bool,
    /// Optional offset for drawing.
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub offset: Vec2,
}

/// Runtime state for a single clip.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ClipState {
    /// Accumulated time since the last frame change.
    pub timer: f32,
    /// Current column index (0‑based).
    pub col: usize,
    /// Current row index (0‑based, relative to the clip’s `rows`).
    pub row: usize,
    /// Whether the clip has finished playing yet.
    pub finished: bool,
}

/// The *single* animation component attached to an entity.
#[derive(Default, Serialize, Deserialize)]
pub struct Animation {
    /// All clips the entity can play.
    pub clips: HashMap<ClipId, Clip>,
    /// Which clip is currently active.
    pub current: ClipId,
    /// Per‑clip runtime data.
    #[serde(skip)]
    pub states: HashMap<ClipId, ClipState>,
}

ecs_component!(Animation);

impl Animation {
    /// Call after deserialization.
    pub fn init_runtime(&mut self) {
        self.states.clear();
        for id in self.clips.keys() {
            self.states.insert(id.clone(), ClipState::default());
        }
    }

    /// Switch to another clip safely.
    pub fn set_clip(&mut self, id: ClipId) {
        if self.clips.contains_key(&id) {
            self.current = id;
            // Reset its timer so the new clip starts from frame 0.
            if let Some(state) = self.states.get_mut(&self.current) {
                *state = ClipState::default();
            }
        }
    }
}
