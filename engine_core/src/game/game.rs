// engine_core/src/game/game.rs
use crate::global::set_global_tile_size;
use crate::game::game_map::GameMap;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;
use crate::{assets::asset_manager::AssetManager, 
    world::world::{World, WorldId}}
;

#[serde_as]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Game {
    pub save_version: u32,
    /// Unique identifier of the game.
    pub id: Uuid,
    /// Human readable name of the game.
    pub name: String,
    /// All worlds belonging to this game instance.
    pub worlds: Vec<World>,
    /// Asset manager for the game.
    pub asset_manager: AssetManager,
    /// Id of the currently active world.
    pub current_world_id: WorldId,
    /// Tile size of the game that the world scales to.
    pub tile_size: f32,
    /// Top level map of the whole game.
    pub game_map: GameMap,
}

impl Game {
    /// Mutable reference to the current world.
    pub fn current_world_mut(&mut self) -> &mut World {
        self.worlds
            .iter_mut()
            .find(|w| w.id == self.current_world_id)
            .expect("Current world id not present in game.")
    }

    /// Immutable reference to the current world.
    pub fn current_world(&self) -> &World {
        self.worlds
            .iter()
            .find(|w| w.id == self.current_world_id)
            .expect("Current world id not present in game.")
    }

    /// Mutable reference to the current world.
    pub fn get_world(&mut self, world_id: WorldId) -> &mut World {
        self.worlds
            .iter_mut()
            .find(|w| w.id == world_id)
            .expect("World id not present in game.")
    }

    /// Add a new world and make it the active one.
    pub fn add_world(&mut self, world: World) {
        self.current_world_id = world.id;
        self.worlds.push(world);
    }

    /// Switch the editor to a different world by its id.
    pub fn select_world(&mut self, id: WorldId) {
        if self.worlds.iter().any(|w| w.id == id) {
            self.current_world_id = id;
        }
    }

    /// Syncs all assets that belong to this game.
    pub async fn initialize(&mut self) {
        set_global_tile_size(self.tile_size);
        let (asset_manager, worlds) = (&mut self.asset_manager, &mut self.worlds);
        asset_manager.init_manager(worlds).await;
    }
}