// game/src/game_state.rs
use crate::scripting::script_system::ScriptSystem;
use engine_core::camera::camera_manager::CameraManager;
use engine_core::storage::core_storage::load_game_ron;
use engine_core::ecs::transform::Transform;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::component::*;
use engine_core::engine_global::*;
use engine_core::world::room::*;
use engine_core::game::game::*;
use std::collections::HashMap;
use bishop::prelude::Vec2;
use mlua::Lua;

/// Top level orchestrator of the game and systems.
pub struct GameState {
    /// The whole game.
    pub game: Game,
    /// Holds the Transform of every entity rendered in the previous frame.
    pub prev_positions: HashMap<Entity, Vec2>,
}

impl GameState {
    // TODO: Make game creation DRYer
    pub async fn new(lua: &Lua, camera_manager: &mut CameraManager) -> Self {
        // Allows the shared engine features to make decisions
        set_engine_mode(EngineMode::Game);

        let mut game = match load_game_ron().await {
            Ok(game) => game,
            Err(e) => panic!("{e}")
        };

        game.initialize(lua).await;

        // TODO: Get rid of expects
        let start_room_id = game.current_world().starting_room_id
            .or_else(|| game.worlds.first().map(|m| m.starting_room_id.expect("Game has no starting room.")))
            .expect("Game has no starting room nor any rooms");

        let current_room = game.current_world().rooms
            .iter()
            .find(|m| m.id == start_room_id)
            .expect("Missing id for the starting room")
            .clone();

        let ecs = &game.ecs;
        let player_pos = ecs.get_player_transform()
            .map(|t| t.position)
            .unwrap_or_default();
        let grid_size = game.current_world().grid_size;
        *camera_manager = CameraManager::new(ecs, current_room.id, player_pos, grid_size);

        ScriptSystem::init(lua);

        Self {
            game,
            prev_positions: HashMap::new(),
        }
    }

    pub async fn for_room(
        room: Room,
        mut game: Game,
        lua: &Lua,
        camera_manager: &mut CameraManager,
        grid_size: f32,
    ) -> Self {
        // Playtest mode is set in playtest_main.rs before this is called,
        // so we only set Game mode if not already in Playtest mode
        if get_engine_mode() != EngineMode::Playtest {
            set_engine_mode(EngineMode::Game);
        }

        game.initialize(lua).await;

        let ecs = &game.ecs;
        let player_pos = ecs.get_player_transform()
            .map(|t| t.position)
            .unwrap_or_default();
        
        *camera_manager = CameraManager::new(ecs, room.id, player_pos, grid_size);

        ScriptSystem::init(lua);

        Self {
            game,
            prev_positions: HashMap::new(),
        }
    }

    /// Updates the previous position for all entities in the active room.
    pub fn store_previous_positions(&mut self, camera_manager: &mut CameraManager) {
        let ecs = &self.game.ecs;
        let trans_store = ecs.get_store::<Transform>();
        let room_store = ecs.get_store::<CurrentRoom>();

        // Store the camera target
        camera_manager.previous_position = Some(camera_manager.active.camera.target);

        self.prev_positions = trans_store.data
            .iter()
            .filter_map(|(entity, transform)| {
                room_store.get(*entity).filter(|cr| cr.0 == self.game.current_world().current_room_id.unwrap()) // TODO: handle unwrap
                    .map(|_| (*entity, transform.position))
            })
            .collect();
    }
}