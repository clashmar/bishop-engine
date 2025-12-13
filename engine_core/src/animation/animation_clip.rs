// engine_core/src/animation/animation_clip.rs
use crate::assets::asset_manager::AssetManager;
use crate::engine_global::tile_size;
use crate::assets::sprite::SpriteId;
use std::{collections::HashMap, path::{Path, PathBuf}};
use serde_with::{FromInto, serde_as};
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use strum_macros::EnumIter;
use macroquad::prelude::*;
use std::fmt;

/// The animation component for an entity.
#[ecs_component(post_create = post_create)]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Animation {
    /// Defineds the animations that belong to the entity.
    pub clips: HashMap<ClipId, ClipDef>,
    /// Which animation variant to show.
    pub variant: VariantFolder,
    /// Which clip is currently active.
    #[serde(skip)]
    pub current: Option<ClipId>,
    /// Per‑clip runtime data.
    #[serde(skip)]
    pub states: HashMap<ClipId, ClipState>,
    /// Cached SpriteId for each clip in the current variant.
    #[serde(skip)]
    pub sprite_cache: HashMap<ClipId, SpriteId>,
}

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

    /// Populate `sprite_cache` for the current variant.
    /// Called when the variant folder changes or a new clip is added.
    pub async fn refresh_sprite_cache(&mut self, asset_manager: &mut AssetManager) {
        self.sprite_cache.clear();

        for (clip_id, _) in &self.clips {
            let sprite_id = resolve_sprite_id(asset_manager, &self.variant, clip_id).await;
            self.sprite_cache.insert(clip_id.clone(), sprite_id);
        }
    }

    /// Creates cache for a clip with a new SpriteId.
    pub fn update_cache_entry(
        &mut self,
        current_id: &ClipId,
        sprite_id: SpriteId,
    ) {
        if sprite_id.0 != 0 {
            self.sprite_cache
                .insert(current_id.clone(), sprite_id);
        }
    }
}

/// Logical name of a clip.
#[derive(EnumIter, Debug, Default, 
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

/// Definition for an animation set.
#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipDef {
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

impl Default for ClipDef {
    fn default() -> ClipDef {
        ClipDef {
            frame_size: vec2(tile_size(), tile_size()),
            cols: 5,
            rows: 1,
            fps: 4.0,
            looping: true,
            offset: Vec2::ZERO,
        }
    }
}

/// A full set of clip definitions that can be reused.
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AnimationDef {
    pub clips: HashMap<ClipId, ClipDef>,
}

/// A variant is a folder that contains the spritesheets for an entity variant.
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VariantFolder(pub PathBuf);

/// Runtime state for a single clip.
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Returns the `SpriteId` for the current variant clip.
pub async fn resolve_sprite_id(
    asset_manager: &mut AssetManager,
    variant_folder: &VariantFolder,
    clip_id: &ClipId,
) -> SpriteId {
    // Build the filename
    let filename = match clip_id {
        ClipId::Idle => "Idle.png",
        ClipId::Walk => "Walk.png",
        ClipId::Run => "Run.png",
        ClipId::Attack => "Attack.png",
        ClipId::Custom(name) => &format!("{}.png", name),
        ClipId::New => unreachable!(),
    };
      
    // Build the path
    let path: PathBuf = Path::new(&variant_folder.0).join(filename);
    
    // Fast‑path if already cached in AssetManager
    if let Some(&id) = asset_manager.path_to_sprite_id.get(&path) {
        return id;
    }

     match asset_manager.init_texture(&path).await {
        Ok(id) => id,
        Err(_) => SpriteId(0) // Sentinal
     }
}

/// Initializes the component when an entity is instantiated into the world.
pub fn post_create(
    anim: &mut Animation
) {
    anim.init_runtime();
}
