// engine_core/src/assets/asset_manager.rs
use crate::animation::animation_clip::Animation;
use crate::storage::path_utils::assets_folder;
use crate::lighting::glow::Glow;
use crate::assets::sprite::*;
use crate::game::game::Game;
use crate::tiles::tile::*;
use serde::{Deserialize, Serialize};
use futures::executor::block_on;
use std::collections::HashSet;
use std::collections::HashMap;
use macroquad::prelude::*;
use std::sync::LazyLock;
use std::path::PathBuf;
use std::path::Path;

#[derive(Serialize, Deserialize, Default)]
pub struct AssetManager {
    /// Name of the game for the file system to use.
    pub game_name: String,
    #[serde(skip)]
    textures: HashMap<SpriteId, Texture2D>,
    /// Persistent map of all sprite is to their paths.
    pub sprite_id_to_path: HashMap<SpriteId, PathBuf>,
    #[serde(skip)]
    pub path_to_sprite_id: HashMap<PathBuf, SpriteId>,
    #[serde(skip)]
    /// Counter for sprite ids. Starts from 1.
    next_sprite_id: usize,
    /// Maps `TileDefIds` to `TileDef`.
    pub tile_defs: HashMap<TileDefId, TileDef>, 
    /// Counter for tile def ids. Starts from 1.
    next_tile_def_id: usize,
}

/// Empty guard texture.
static EMPTY_TEXTURE: LazyLock<Texture2D> = LazyLock::new(|| Texture2D::empty());

impl AssetManager {
    /// Initializes a new asset manager.
    pub async fn new(game_name: String) -> Self {
        Self {
            game_name,
            textures: HashMap::new(),
            path_to_sprite_id: HashMap::new(),
            sprite_id_to_path: HashMap::new(),
            next_sprite_id: 1,
            tile_defs: HashMap::new(),
            next_tile_def_id: 1,
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

        // Set and calculate the next texture id
        let id = SpriteId(self.next_sprite_id);
        self.restore_next_sprite_id();

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
    pub async fn init_manager(game: &mut Game) {
        // Calculate the next id from the existing map
        game.asset_manager.restore_next_sprite_id();

        // Restore next tile def id
        if let Some(max_id) = game.asset_manager.tile_defs.keys().map(|id| id.0).max() {
            game.asset_manager.next_tile_def_id = max_id + 1;
        } else {
            game.asset_manager.next_tile_def_id = 1;
        }

        let _purged = Self::purge_unused_assets(game);

        let sprites: Vec<(SpriteId, PathBuf)> = game.asset_manager
            .sprite_id_to_path
            .iter()
            .map(|(id, path)| (*id, path.clone()))
            .collect();

        // Reload all textures
        for (id, path) in sprites {
            let _ = game.asset_manager.reload_texture(&id, &path).await;
        }

        // Load and initialize all animations
        for animation in game.ecs.get_store_mut::<Animation>().data.values_mut() {
            animation.refresh_sprite_cache(&mut game.asset_manager).await;
            animation.init_runtime();
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

    /// Calculates the next sprite id.
    pub fn restore_next_sprite_id(&mut self) {
        let used: HashSet<_> = self.sprite_id_to_path
            .keys()
            .map(|sid| sid.0)
            .collect();

        let mut candidate = 1usize;

        // Scan through until an unused id is found
        while used.contains(&candidate) {
            candidate += 1;
        }

        self.next_sprite_id = candidate;
    }

    /// Inserts a TileDef and returns its id.
    pub fn insert_tile_def(&mut self, def: TileDef) -> TileDefId {
        let id = TileDefId(self.next_tile_def_id);
        self.next_tile_def_id += 1;
        self.tile_defs.insert(id, def);
        id
    }

    /// Removes all sprite ids that are no longer referenced by any loaded world.
    /// Returns the number of ids that were purged. 
    /// Only call this on program init/close to protect the undo/redo stack.
    pub fn purge_unused_assets(game: &mut Game) -> usize {
        // Collect every SpriteId that is still in use
        let mut used_ids: HashSet<SpriteId> = HashSet::new();

        // TODO purge all other assets from the game when they exist

        // Tiles
        for tile_def in game.asset_manager.tile_defs.values() {
            if tile_def.sprite_id.0 != 0 {
                used_ids.insert(tile_def.sprite_id);
            }
        }

        for world in &game.worlds {
            if let Some(id) = world.meta.sprite_id {
                used_ids.insert(id);
            }
        }

        // Sprite components
        let sprite_store = game.ecs.get_store::<Sprite>();
        for sprite in sprite_store.data.values() {
            if sprite.sprite.0 != 0 {
                used_ids.insert(sprite.sprite);
            }
        }

        // Glow components
        let glow_store = game.ecs.get_store::<Glow>();
        for glow in glow_store.data.values() {
            if glow.sprite_id.0 != 0 {
                used_ids.insert(glow.sprite_id);
            }
        }

        // Animation component caches (should be full after initialization)
        let anim_store = game.ecs.get_store::<Animation>();
        for anim in anim_store.data.values() {
            for &id in anim.sprite_cache.values() {
                if id.0 != 0 {
                    used_ids.insert(id);
                }
            }
        }
        

        // Capture the current number of sprite ids
        let previous = game.asset_manager.sprite_id_to_path.len();

        // Closure which keeps entries that are still in use
        let keep = |id: &SpriteId| used_ids.contains(id);

        // Remove stale textures
        game.asset_manager.textures.retain(|id, _| keep(id));

        // Remove stale paths
        game.asset_manager.path_to_sprite_id.retain(|_, id| keep(id));

        // Remove stale ids
        game.asset_manager.sprite_id_to_path.retain(|id, _| keep(id));

        // Calculate the next free id
        game.asset_manager.restore_next_sprite_id();

        // Return the number of purged ids
        previous - game.asset_manager.sprite_id_to_path.len()
    }
}