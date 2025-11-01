// engine_core/src/assets/asset_manager.rs
use std::path::Path;
use futures::executor::block_on;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{
    animation::animation_clip::Animation, assets::sprite::SpriteId, world::world::World
};

#[derive(Serialize, Deserialize, Default)]
pub struct AssetManager {
    #[serde(skip)]
    textures: HashMap<SpriteId, Texture2D>,
    #[serde(skip)]
    pub path_to_sprite_id: HashMap<String, SpriteId>,
    pub sprite_id_to_path: HashMap<SpriteId, String>,
    #[serde(skip)]
    /// Counter for sprite ids. Starts from 1.
    next_sprite_id: usize,
}

impl AssetManager {
    /// Initializes a new asset manager with all sprite textures loaded.
    pub async fn new() -> Self {
        Self {
            textures: HashMap::new(),
            path_to_sprite_id: HashMap::new(),
            sprite_id_to_path: HashMap::new(),
            next_sprite_id: 1,
        }
    }

    /// Load and initialize a texture from the assets folder.
    /// Returns the `SpriteId` for the texture.
    pub async fn init_texture(&mut self, rel_path: impl AsRef<Path>) -> Result<SpriteId, String> {
        let key = rel_path.as_ref().to_string_lossy().to_string();

        if key.trim().is_empty() {
            // Guard against path being empty
            return Err("Empty texture path".into());
        }

        // Already loaded, reuse the same id
        if let Some(&id) = self.path_to_sprite_id.get(&key) {
            return Ok(id);
        }

        // Load the texture from disk.
        let texture = load_texture(&key)
            .await
            .map_err(|e| format!("Failed to load texture '{}': {}", key, e))?;

        // Disable smoothing (needed for pixel art)
        texture.set_filter(FilterMode::Nearest);

        // Set and increment the texture id
        let id = SpriteId(self.next_sprite_id);
        self.next_sprite_id += 1;

        // Store everything
        self.textures.insert(id, texture);
        self.path_to_sprite_id.insert(key.clone(), id);
        self.sprite_id_to_path.insert(id, key);

        return Ok(id);
    }

    /// Reloads a texture from its `SpriteId` and updates `path_to_sprite_id`.
    pub async fn reload_texture(&mut self, id: &SpriteId, path: &String) -> Result<(), String> {
        // Load the texture from disk.
        let texture = load_texture(path)
            .await
            .map_err(|e| format!("Failed to load texture '{}': {}", path, e))?;

        // Disable smoothing (needed for pixel art)
        texture.set_filter(FilterMode::Nearest);

        // Store everything and repopulate the reverse map
        self.textures.insert(*id, texture);
        self.path_to_sprite_id.insert(path.clone(), *id);

        return Ok(());
    }

    /// Returns a texture from a `SpriteId`. If the texture has not been loaded yet load it synchronously.
    pub fn get_texture_from_id(&mut self, id: SpriteId) -> &Texture2D {
        // Fast path
        if self.contains(id) {
            return self.textures.get(&id).unwrap();
        }

        // Look up the original path and load it now.
        let path = self
            .sprite_id_to_path
            .get(&id)
            .expect("SpriteId out of range and no stored path")
            .clone();

        let _ = block_on(self.init_texture(path));
        self.textures.get(&id).unwrap()
    }

    /// Returns the id for `path`, loading it if necessary.
    pub fn get_or_load(&mut self, path: &str) -> Option<SpriteId> {
        if path.trim().is_empty() {
            return None;
        }

        if let Some(&id) = self.path_to_sprite_id.get(path) {
            return Some(id);
        }

        // Blocking load
        match block_on(self.init_texture(path)) {
            Ok(id) => Some(id),
            Err(err) => {
                info!("{}", err);
                None
            }
        }
    }

    /// Returns the id for `path` or `None` if not loaded.
    pub fn get_or_none(&self, path: &str) -> Option<SpriteId> {
        if path.trim().is_empty() {
            return None;
        }

        if let Some(&id) = self.path_to_sprite_id.get(path) {
            return Some(id)
        }

        None
    }

    /// Initialize all assets for the game.
    pub async fn init(&mut self, worlds: &mut Vec<World>) {
        // Calculate the next id from the existing map
        self.restore_next_id();

        let sprites: Vec<(SpriteId, String)> = self
            .sprite_id_to_path
            .iter()
            .map(|(id, path)| (*id, path.clone()))
            .collect();

        // Reload all textures
        for (id, path) in sprites {
            let _ = self.reload_texture(&id, &path).await;
        }

        for world in worlds {
            let world_ecs = &mut world.world_ecs;

            // Load and initialize all animations
            for animation in world_ecs.get_store_mut::<Animation>().data.values_mut() {
                animation.refresh_sprite_cache(self).await;
                animation.init_runtime();
            }
        }
    }

    /// Returns true if the texture for `id` is already present.
    #[inline]
    pub fn contains(&self, id: SpriteId) -> bool {
        self.textures.contains_key(&id)
    }

    /// Return the pixel width and height of the texture that belongs to `id`
    /// or None if the texture has not been loaded/set.
    pub fn texture_size(&self, id: SpriteId) -> Option<(f32, f32)> {
        self.textures.get(&id).map(|tex| (tex.width(), tex.height()))
    }

    fn restore_next_id(&mut self) {
        if let Some(max_id) = self.sprite_id_to_path.keys().map(|id| id.0).max() {
            self.next_sprite_id = max_id + 1;
        } else {
            self.next_sprite_id = 1;
        }
    }
}