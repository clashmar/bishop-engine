// engine_core/src/assets/asset_manager.rs
use std::{path::{Path, PathBuf}, sync::LazyLock};
use futures::executor::block_on;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{
    animation::animation_clip::Animation, assets::sprite::SpriteId, storage::path_utils::assets_folder, world::world::World
};

#[derive(Serialize, Deserialize, Default)]
pub struct AssetManager {
    #[serde(skip)]
    textures: HashMap<SpriteId, Texture2D>,
    /// Persistent map of all sprite is to their paths.
    pub sprite_id_to_path: HashMap<SpriteId, PathBuf>,
    #[serde(skip)]
    pub path_to_sprite_id: HashMap<PathBuf, SpriteId>,
    #[serde(skip)]
    /// Counter for sprite ids. Starts from 1.
    next_sprite_id: usize,
    /// Name of the game for the file system to use.
    pub game_name: String,
}

/// Empty texture which guards against crashes.
static EMPTY_TEXTURE: LazyLock<Texture2D> = LazyLock::new(|| Texture2D::empty());

impl AssetManager {
    /// Initializes a new asset manager.
    pub async fn new(game_name: String) -> Self {
        Self {
            textures: HashMap::new(),
            path_to_sprite_id: HashMap::new(),
            sprite_id_to_path: HashMap::new(),
            next_sprite_id: 1,
            game_name,
        }
    }

    /// Load and initialize a texture from the assets folder.
    /// Returns the `SpriteId` for the texture.
    pub async fn init_texture(&mut self, rel_path: impl AsRef<Path>) -> Result<SpriteId, String> {
        let path = rel_path.as_ref().to_path_buf();

        if path.to_string_lossy().trim().is_empty() {
            // Guard against path being empty
            return Err("Empty texture path".into());
        }

        // Already loaded, reuse the same id
        if let Some(&id) = self.path_to_sprite_id.get(&path) {
            return Ok(id);
        }

        // Load the texture from the assets folder.
        let texture = self.load_texture_from_game(&path).await?;

        // Set and increment the texture id
        let id = SpriteId(self.next_sprite_id);
        self.next_sprite_id += 1;

        // Store everything
        self.textures.insert(id, texture);
        self.path_to_sprite_id.insert(path.clone(), id);
        self.sprite_id_to_path.insert(id, path);

        return Ok(id);
    }

    /// Reloads a texture from its `SpriteId` and updates `path_to_sprite_id`.
    pub async fn reload_texture(&mut self, id: &SpriteId, path: &Path) -> Result<(), String> {
        // Load the texture from disk.
        let texture = self.load_texture_from_game(&path).await?;

        // Store everything and repopulate the reverse map
        self.textures.insert(*id, texture);
        self.path_to_sprite_id.insert(path.to_path_buf(), *id);

        return Ok(());
    }

    /// Returns a texture from a `SpriteId`. If the texture has not been loaded yet load it synchronously.
    pub fn get_texture_from_id(&mut self, id: SpriteId) -> &Texture2D {
        // If SpriteId = 0 it is unset
        if id.0 == 0 {
            return &*EMPTY_TEXTURE;
        }

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
    pub fn get_or_load<P: AsRef<Path>>(&mut self, path: P) -> Option<SpriteId> {
        let p = path.as_ref();
        if p.to_string_lossy().trim().is_empty() {
            return None;
        }

        if let Some(&id) = self.path_to_sprite_id.get(p) {
            return Some(id);
        }

        // Blocking load
        match block_on(self.init_texture(p)) {
            Ok(id) => Some(id),
            Err(err) => {
                info!("{}", err);
                None
            }
        }
    }

    /// Returns the id for `path` or `None` if not loaded.
    pub fn get_or_none<P: AsRef<Path>>(&self, path: P) -> Option<SpriteId> {
        let p = path.as_ref();
        if p.to_string_lossy().trim().is_empty() {
            return None;
        }
        if let Some(&id) = self.path_to_sprite_id.get(p) {
            return Some(id);
        }
        None
    }

    /// Initialize all assets for the game.
    pub async fn init_manager(&mut self, worlds: &mut Vec<World>) {
        // Calculate the next id from the existing map
        self.restore_next_id();

        let sprites: Vec<(SpriteId, PathBuf)> = self
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

    /// Returns a path normalized relative to the game's assets folder.
    pub fn normalize_path(&self, path: PathBuf) -> PathBuf {
        let assets_dir = assets_folder(&self.game_name);
        path.strip_prefix(&assets_dir)
            .unwrap_or_else(|_| &path)
            .to_path_buf()
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

    /// Loads a texture from the assets folder.
    async fn load_texture_from_game<P: AsRef<Path> + Copy>(
        &self,
        rel_path: P,
    ) -> Result<Texture2D, String> {
        let full_path = assets_folder(&self.game_name).join(rel_path);

        load_texture(full_path.to_string_lossy().as_ref())
            .await
            .map(|texture| {
                // Disable smoothing (needed for pixel art)
                texture.set_filter(FilterMode::Nearest);
                texture
            })
            .map_err(|e| {
                format!(
                    "Failed to load texture '{}': {}",
                    rel_path.as_ref().display(),
                    e
                )
            })
    }

    /// Calculates the next sprite id 
    fn restore_next_id(&mut self) {
        if let Some(max_id) = self.sprite_id_to_path.keys().map(|id| id.0).max() {
            self.next_sprite_id = max_id + 1;
        } else {
            self.next_sprite_id = 1;
        }
    }
}