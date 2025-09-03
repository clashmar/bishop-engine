use std::path::Path;
use macroquad::prelude::*;
use std::collections::HashMap;
use crate::assets::sprites::SpriteId;

pub struct AssetManager {
    textures: Vec<Texture2D>,
    path_to_id: HashMap<String, SpriteId>,
    id_to_path: Vec<String>, 
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            path_to_id: HashMap::new(),
            id_to_path: Vec::new(),
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

        // New id is the next free index.
        let id = SpriteId(self.textures.len());

        // Store everything.
        self.textures.push(texture);
        self.path_to_id.insert(key.clone(), id);
        self.id_to_path.push(key); // keep the reverse lookup

        id
    }

    /// Returns true if the texture for `id` is already present.
    #[inline]
    pub fn contains(&self, id: SpriteId) -> bool {
        id.0 < self.textures.len()
    }

    /// If the texture has not been loaded yet load it synchronously.
    pub fn get(&mut self, id: SpriteId) -> &Texture2D {
        // Fast path
        if self.contains(id) {
            return &self.textures[id.0];
        }

        // Look up the original path and load it now.
        let path: String = self
            .id_to_path
            .get(id.0)
            .expect("SpriteId out of range and no stored path")
            .clone();

        futures::executor::block_on(self.load(path));
        &self.textures[id.0]
    }
}