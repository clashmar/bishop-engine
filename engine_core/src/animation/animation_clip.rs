// engine_core/src/animation/animation_clip.rs
use strum_macros::{EnumIter, EnumString};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use serde_with::{FromInto, serde_as};
use std::fmt;

use crate::{assets::sprite::SpriteId, constants::TILE_SIZE, ecs_component};

/// Logical name of a clip.
#[derive(EnumIter, EnumString, Debug, Default, 
    Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClipId {
    #[default]
    Idle,
    Walk,
    Run,
    Attack,
    Custom(String),
    New,
}

impl ClipId {
    /// Returns the text that should be shown in dropdowns, lists, etc.
    pub fn ui_label(&self) -> String {
        match self {
            // Empty
            ClipId::New => "New".to_string(),
            // Any non‑empty custom name
            ClipId::Custom(name) => name.clone(),
            // Built‑in variants
            _ => format!("{self:?}"),
        }
    }
}

impl fmt::Display for ClipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ui_label())
    }
}

/// One animation clip inside a sprite sheet.
#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct Clip {
    /// Texture that holds the sheet.
    pub sprite_id: SpriteId,
    /// Path that was used to load the texture.
    pub path: String,
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

impl Default for Clip {
    fn default() -> Clip {
        Clip {
            sprite_id: SpriteId(Uuid::nil()),
            path: String::new(),
            frame_size: vec2(TILE_SIZE, TILE_SIZE),
            cols: 5,
            rows: 1,
            fps: 4.0,
            looping: true,
            offset: Vec2::ZERO,
        }
    }
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
    pub current: Option<ClipId>,
    /// Per‑clip runtime data.
    #[serde(skip)]
    pub states: HashMap<ClipId, ClipState>,
}

ecs_component!(Animation);

impl Animation {
    /// Call after deserialization or after a clip has been added/removed.
    pub fn init_runtime(&mut self) {
        self.states.clear();
        for id in self.clips.keys() {
            self.states.insert(id.clone(), ClipState::default());
        }

        // If there is at least one clip but `current` is None, pick the first
        if self.current.is_none() && !self.clips.is_empty() {
            self.current = Some(self.clips.keys().next().unwrap().clone());
        }
    }

    /// Switch to another clip safely.
    pub fn set_clip(&mut self, id: &ClipId) {
        if self.clips.contains_key(&id) {
            self.current = Some(id.clone());
            // Reset its timer so the new clip starts from frame 0.
            if let Some(state) = self.states.get_mut(&id) {
                *state = ClipState::default();
            }
        }
    }

    /// Clear the active clip.
    pub fn clear_clip(&mut self) {
        self.current = None;
    }
}
