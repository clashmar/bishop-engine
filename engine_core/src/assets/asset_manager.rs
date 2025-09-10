// engine_core/src/assets/asset_manager.rs
use std::path::Path;
use macroquad::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use crate::assets::sprite::SpriteId;

pub struct AssetManager {
    textures: HashMap<SpriteId, Texture2D>,
    path_to_id: HashMap<String, SpriteId>,
    id_to_path: HashMap<SpriteId, String>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
        }
    }

    /// Load a texture from the assets folder.
    /// Returns the `SpriteId` that can later be used with `get`.
    pub async fn load(&mut self, rel_path: impl AsRef<Path>) -> SpriteId {
        let key = rel_path.as_ref().to_string_lossy().to_string();

        // Already loaded, reuse the same id
        if let Some(&id) = self.path_to_id.get(&key) {
            return id;
        }

        // Load the texture from disk.
        let texture = load_texture(&key)
            .await
            .expect("Could not load texture.");

        // Disable smoothing (needed for pixel art)
        texture.set_filter(FilterMode::Nearest);

        // Create a fresh UUID for this texture
        let id = SpriteId(Uuid::new_v4());

        // Store everything
        self.textures.insert(id, texture);
        self.path_to_id.insert(key.clone(), id);
        self.id_to_path.insert(id, key);
        id
    }

    /// Returns true if the texture for `id` is already present.
    #[inline]
    pub fn contains(&self, id: SpriteId) -> bool {
        self.textures.contains_key(&id)
    }

    /// Returns a texture from a sprite id. If the texture has not been loaded yet load it synchronously.
    pub fn get_texture_from_id(&mut self, id: SpriteId) -> &Texture2D {
        // Fast path
        if self.contains(id) {
            return self.textures.get(&id).unwrap();
        }

        // Look up the original path and load it now.
        let path = self
            .id_to_path
            .get(&id)
            .expect("SpriteId out of range and no stored path")
            .clone();
        futures::executor::block_on(self.load(path));
        self.textures.get(&id).unwrap()
    }
}