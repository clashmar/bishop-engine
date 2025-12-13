// engine_core/src/game/game.rs
use crate::{ecs::world_ecs::WorldEcs, world::room::Room};
use crate::scripting::script_manager::ScriptManager;
use crate::assets::asset_manager::AssetManager;
use crate::engine_global::set_global_tile_size;
use crate::game::game_map::GameMap;
use crate::world::world::WorldId;
use crate::world::world::World;
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

/// Temporary view into a `Game` that bundles together the 
/// immutable systems that are usually needed at the same time.
pub struct GameCtx<'a> {
    // TODO: wrap in options
    pub cur_world_ecs: &'a WorldEcs,
    pub cur_room: &'a Room,
    pub asset_manager: &'a AssetManager,
    pub script_manager: &'a ScriptManager,
}

/// Temporary view into a `Game` that bundles together the 
/// mutable systems that are usually needed at the same time.
pub struct GameCtxMut<'a> {
    // TODO: wrap in options
    pub cur_world_ecs: &'a mut WorldEcs,
    pub cur_room: &'a mut Room,
    pub asset_manager: &'a mut AssetManager,
    pub script_manager: &'a mut ScriptManager,
}

impl Game {
    /// Returns an immutable game context.
    pub fn ctx<'a>(&'a self) -> GameCtx<'a> {
        let world = self
            .worlds
            .iter()
            .find(|w| w.id == self.current_world_id)
            .expect("There must be a current world.");

        let cur_world_ecs = &world.world_ecs;
        let rooms = &world.rooms;

        let room_id = world.current_room_id.expect("Room id not found.");

        let cur_room = rooms
            .iter()
            .find(|r| r.id == room_id)
            .expect("Room not found.");

        GameCtx {
            cur_world_ecs,
            cur_room,
            asset_manager: &self.asset_manager,
            script_manager: &self.script_manager,
        }
    }

    /// Returns a mutable game context.
    pub fn ctx_mut<'a>(&'a mut self) -> GameCtxMut<'a> {
        let world = self
            .worlds
            .iter_mut()
            .find(|w| w.id == self.current_world_id)
            .expect("There must be a current world.");

        // First borrow into separate disjoint fields
        let cur_world_ecs = &mut world.world_ecs;
        let rooms = &mut world.rooms;

        // Now you can borrow a room
        let room_id = world.current_room_id.expect("Room id not found.");

        let cur_room = rooms
            .iter_mut()
            .find(|r| r.id == room_id)
            .expect("Room not found.");

        GameCtxMut {
            cur_world_ecs,
            cur_room,
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
        set_global_tile_size(self.tile_size);
        AssetManager::init_manager(self).await;
        ScriptManager::init_manager(self, lua).await;
    }
}