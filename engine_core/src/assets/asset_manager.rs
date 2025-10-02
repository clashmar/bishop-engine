// engine_core/src/assets/asset_manager.rs
use std::path::Path;
use macroquad::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{
    animation::animation_clip::Animation, assets::sprite::{Sprite, SpriteId}, ecs::world_ecs::WorldEcs, tiles::tile::TileSprite
};

pub struct AssetManager {
    textures: HashMap<SpriteId, Texture2D>,
    pub path_to_id: HashMap<String, SpriteId>,
    id_to_path: HashMap<SpriteId, String>,
}

impl AssetManager {
    /// Initializes a new asset manager with all sprite textures loaded.
    pub async fn new(world_ecs: &mut WorldEcs) -> Self {
        let mut asset_manager = Self {
            textures: HashMap::new(),
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
        };

        asset_manager.sync_all_assets(world_ecs).await;
        asset_manager
    }

    /// Load a texture from the assets folder.
    /// Returns the `SpriteId` that can later be used with `get`.
    pub async fn load(&mut self, rel_path: impl AsRef<Path>) -> SpriteId {
        let key = rel_path.as_ref().to_string_lossy().to_string();

        if key.trim().is_empty() {
            // Guard against path being empty
            return SpriteId(Uuid::nil());
        }

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

    /// Syncs all the sprite assets for a world.
    pub async fn sync_all_assets(&mut self, world_ecs: &mut WorldEcs) {
        // Load all non‑tile sprites
        for (_entity, sprite) in world_ecs.get_store_mut::<Sprite>().data.iter_mut() {
            if !self.contains(sprite.sprite_id) {
                let id = self.load(&sprite.path).await;
                sprite.sprite_id = id;
            }
        }

        // Load all tile‑sprites
        for (_entity, tile_sprite) in world_ecs.get_store_mut::<TileSprite>().data.iter_mut() {
            if !self.contains(tile_sprite.sprite_id) {
                let id = self.load(&tile_sprite.path).await;
                tile_sprite.sprite_id = id;
            }
        }

        // Load and initialize all animations
        for animation in world_ecs.get_store_mut::<Animation>().data.values_mut() {
            animation.refresh_sprite_cache(self).await;
            animation.init_runtime();
        }
    }

    /// Return the pixel width and height of the texture that belongs to `id`
    /// or None if the texture has not been loaded/set.
    pub fn texture_size(&self, id: SpriteId) -> Option<(f32, f32)> {
        self.textures.get(&id).map(|tex| (tex.width(), tex.height()))
    }
}