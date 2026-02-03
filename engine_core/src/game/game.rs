// engine_core/src/game/game.rs
use crate::scripting::script_manager::ScriptManager;
use crate::assets::asset_manager::AssetManager;
use crate::game::game_map::GameMap;
use crate::engine_global::*;
use crate::world::world::*;
use crate::ecs::ecs::Ecs;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;
use mlua::Lua;

#[serde_as]
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Game {
    pub save_version: u32,
    /// Unique identifier of the game.
    pub id: Uuid,
    /// Human readable name of the game.
    pub name: String,
    /// Stores the game ECS.
    pub ecs: Ecs,
    /// All worlds belonging to this game instance.
    pub worlds: Vec<World>,
    /// Asset manager for the game.
    pub asset_manager: AssetManager,
    /// Script manager for the game.
    pub script_manager: ScriptManager,
    /// Id of the currently active world.
    pub current_world_id: WorldId, // TODO: Change this to an option
    /// Tile size of the game that the world scales to.
    pub tile_size: f32,
    /// Top level map of the whole game.
    pub game_map: GameMap,
}

/// Bundles together common immutable systems.
pub struct GameCtx<'a> {
    pub ecs: &'a Ecs,
    pub cur_world: &'a World,
    pub asset_manager: &'a AssetManager,
    pub script_manager: &'a ScriptManager,
}

/// Bundles together common mutable systems.
pub struct GameCtxMut<'a> {
    pub ecs: &'a mut Ecs,
    pub cur_world: &'a mut World,
    pub asset_manager: &'a mut AssetManager,
    pub script_manager: &'a mut ScriptManager,
}

impl Game {
    /// Returns an immutable game context.
    pub fn ctx<'a>(&'a self) -> GameCtx<'a> {
        let cur_world = self
            .worlds
            .iter()
            .find(|w| w.id == self.current_world_id)
            .expect("There must be a current world.");

        GameCtx {
            ecs: &self.ecs,
            cur_world,
            asset_manager: &self.asset_manager,
            script_manager: &self.script_manager,
        }
    }

    /// Returns a mutable game context.
    pub fn ctx_mut<'a>(&'a mut self) -> GameCtxMut<'a> {
        let cur_world = self
            .worlds
            .iter_mut()
            .find(|w| w.id == self.current_world_id)
            .expect("There must be a current world.");

        GameCtxMut {
            ecs: &mut self.ecs,
            cur_world,
            asset_manager: &mut self.asset_manager,
            script_manager: &mut self.script_manager,
        }
    }

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

    /// Gets a mutable reference to a world from its id.
    pub fn get_world_mut(&mut self, world_id: WorldId) -> &mut World {
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

    /// Deletes the world from the game.
    pub fn delete_world(&mut self, id: WorldId) {
        if let Some(pos) = self.worlds.iter().position(|w| w.id == id) {
            self.worlds.swap_remove(pos);
        }

        if self.current_world_id == id {
            self.current_world_id = self.worlds
                .first()
                .map(|w| w.id)
                .unwrap_or(WorldId(Uuid::nil()));
        }
    }

    /// Syncs all assets/scripts that belong to this game, sets the global tile size and inits input.
    pub async fn initialize(&mut self, lua: &Lua) {
        set_game_name(self.name.clone());
        set_global_tile_size(self.tile_size);
        AssetManager::init_manager(self).await;
        ScriptManager::init_manager(self, lua).await;
    }
}