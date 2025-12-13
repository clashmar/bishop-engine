// game/src/game_state.rs
use crate::scripting::script_system::ScriptSystem;
// use engine_core::world::transition_manager::TransitionManager;
use engine_core::camera::camera_manager::CameraManager;
use engine_core::storage::core_storage::load_game_ron;
use engine_core::ecs::component::CurrentRoom;
use engine_core::ecs::component::Position;
use engine_core::ecs::entity::Entity;
use engine_core::world::room::Room;
use engine_core::engine_global::*;
use engine_core::game::game::*;
use std::collections::HashMap;
use macroquad::prelude::*;
use std::cell::RefCell;
use mlua::Value;
use mlua::Lua;

/// Top level orchestrator of the game and systems.
pub struct GameState {
    /// The whole game.
    pub game: Game,
    /// Manages transitions between rooms.
    // transition_manager: TransitionManager,
    /// Current room
    pub current_room: Room,
    /// TODO: ???
    pub global_modules: RefCell<HashMap<String, Value>>,
    /// Holds the Position of every entity rendered in the previous frame.
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

        let world_ecs = &game.current_world().world_ecs;
        let player_pos = world_ecs.get_player_position().position;
        *camera_manager = CameraManager::new(world_ecs, current_room.id, player_pos);

        ScriptSystem::init(lua, &mut game.script_manager);

        Self {
            game,
            // transition_manager: TransitionManager::new(),
            current_room,
            global_modules: RefCell::new(HashMap::new()),
            prev_positions: HashMap::new(),
        }
    }

    pub async fn for_room(
        room: Room, 
        mut game: Game, 
        lua: &Lua,
        camera_manager: &mut CameraManager,
    ) -> Self {
        // Allows the shared engine features to make decisions
        // set_engine_mode(EngineMode::Game); TODO: figure this out

        game.initialize(lua).await;
        let world_ecs = &game.current_world().world_ecs;
        let player_pos = world_ecs.get_player_position().position;
        *camera_manager = CameraManager::new(world_ecs, room.id, player_pos);

        ScriptSystem::init(lua, &mut game.script_manager);

        Self {
            game,
            // transition_manager: TransitionManager::new(),
            current_room: room,
            global_modules: RefCell::new(HashMap::new()),
            prev_positions: HashMap::new(),
        }
    }

    /// Updates the previous position for all entities in the active room.
    pub fn store_previous_positions(&mut self, camera_manager: &mut CameraManager) {
        let current_world = self.game.current_world_mut();

        let pos_store = current_world.world_ecs.get_store::<Position>();
        let room_store = current_world.world_ecs.get_store::<CurrentRoom>();

        // Store the camera target
        camera_manager.previous_position = Some(camera_manager.active.camera.target);

        self.prev_positions = pos_store.data
            .iter()
            .filter_map(|(entity, pos)| {
                room_store.get(*entity).filter(|cr| cr.0 == self.current_room.id)
                    .map(|_| (*entity, pos.position))
            })
            .collect();
    }
}