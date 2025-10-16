// engine_core/src/game/game.rs
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;
use crate::
    world::world::World
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
    /// Id of the currently active world.
    pub current_world_id: Uuid,
    /// Tile size of the game that the world scales to.
    pub tile_size: f32,
}

impl Game {
    /// Mutable reference to the world the editor is currently editing.
    pub fn current_world_mut(&mut self) -> &mut World {
        self.worlds
            .iter_mut()
            .find(|w| w.id == self.current_world_id)
            .expect("Current world UUID not present in game.")
    }

    /// Immutable reference to the current world.
    pub fn current_world(&self) -> &World {
        self.worlds
            .iter()
            .find(|w| w.id == self.current_world_id)
            .expect("Current world UUID not present in game.")
    }

    /// Add a new world and make it the active one.
    pub fn add_world(&mut self, world: World) {
        self.current_world_id = world.id;
        self.worlds.push(world);
    }

    /// Switch the editor to a different world by its UUID.
    pub fn select_world(&mut self, id: Uuid) {
        if self.worlds.iter().any(|w| w.id == id) {
            self.current_world_id = id;
        }
    }
}