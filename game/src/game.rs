// game/src/game.rs
use engine_core::*;
use crate::input::input_system::*;
use crate::physics::physics_system::*;
use engine_core::global::*;
use engine_core::onscreen_error;
use engine_core::rendering::render_room::*;
use engine_core::animation::animation_system::*;
use engine_core::ecs::component::Position;
use engine_core::ecs::component::CurrentRoom;
use engine_core::constants::*;
use engine_core::ecs::entity::Entity;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::script::script_system::run_scripts;
use engine_core::storage::core_storage::load_game_ron;
use engine_core::world::room::Room;
use engine_core::world::transition_manager::TransitionManager;
use engine_core::camera::camera_manager::CameraManager;
use engine_core::game::game::*;
use std::collections::HashMap;
use macroquad::prelude::*;

pub struct GameState {
    /// The whole game.
    game: Game,
    /// Camera that follows the player.
    camera_manager: CameraManager,
    /// Manages transitions between rooms.
    transition_manager: TransitionManager,
    /// Current room
    current_room: Room,
    /// Rendering system for the game.
    pub render_system: RenderSystem,
    /// Holds the Position of every entity rendered in the previous frame.
    prev_positions: HashMap<Entity, Vec2>,
}

impl GameState {
    pub async fn new() -> Self {
        // Allows the shared engine features to make decisions
        set_engine_mode(EngineMode::Game);
        
        let game = match load_game_ron().await {
            Ok(game) => game,
            Err(e) => panic!("{e}")
        };

        // TODO: Get rid of expects
        let start_room_id = game.current_world().starting_room_id
            .or_else(|| game.worlds.first().map(|m| m.starting_room_id.expect("Game has no starting room.")))
            .expect("Game has no starting room nor any rooms");

        let current_room = game.current_world().rooms
            .iter()
            .find(|m| m.id == start_room_id)
            .expect("Missing id for the starting room")
            .clone();

        let player_pos = game.current_world().world_ecs.get_player_position().position;
        let camera_manager = CameraManager::new(&game.current_world().world_ecs, current_room.id, player_pos);

        Self {
            game,
            camera_manager,
            transition_manager: TransitionManager::new(),
            current_room,
            render_system: RenderSystem::new(),
            prev_positions: HashMap::new(),
        }
    }

    pub async fn for_room(room: Room, mut game: Game) -> Self {
        game.initialize().await;
        let world_ecs = &mut game.current_world_mut().world_ecs;
        let player_pos = world_ecs.get_player_position().position;
        let camera_manager = CameraManager::new(world_ecs, room.id, player_pos);

        Self {
            game,
            camera_manager,
            transition_manager: TransitionManager::new(),
            current_room: room,
            render_system: RenderSystem::new(),
            prev_positions: HashMap::new(),
        }
    }

    pub async fn run_game_loop(&mut self) {
        let mut accumulator: f32 = 0.0;
        let mut cur_window_size = (screen_width() as u32, screen_height() as u32);

        // Main loop
        loop {
            // Time elapsed since last frame
            let frame_dt = get_frame_time();
            accumulator = (accumulator + frame_dt).min(MAX_ACCUM);
            
            // Input system
            self.poll_input();
            
            // Fixed‑step physics
            while accumulator >= FIXED_DT {
                self.fixed_update(FIXED_DT);
                accumulator -= FIXED_DT;
            }
            
            // Per‑frame async work (input, animation, camera …)
            self.update_async(frame_dt).await;

            // Render with interpolation
            let alpha = accumulator / FIXED_DT;
            self.render(alpha, &mut cur_window_size);

            next_frame().await;
        }
    }

    pub fn poll_input(&mut self) {
        update_player_input(&mut self.game)
    } 

    pub fn fixed_update(&mut self, dt: f32) {
        // Store the current positions for the next frame
        self.store_previous_positions();

        let current_world = self.game.current_world_mut();

        let player = current_world.world_ecs.get_player_entity();

        // If an entity exits the current room TODO: Decouple room transitions from physics
        if let Some((exiting_entity, target_id, new_pos)) = 
            update_physics(
                &mut current_world.world_ecs, 
                &self.current_room, 
                dt
            ) {
            let new_room = current_world
                .rooms
                .iter()
                .find(|r| r.id == target_id)
                .expect("Target room not found");

            // Only update the game current room if the player exits
            if exiting_entity == player {
                self.current_room = new_room.clone();
            }

            let cur_room_mut = current_world.world_ecs.get_mut::<CurrentRoom>(exiting_entity).unwrap();
            cur_room_mut.0 = new_room.id;

            let pos_mut = current_world.world_ecs.get_mut::<Position>(exiting_entity).unwrap();
            pos_mut.position = new_pos;
        }
    }

    pub async fn update_async(&mut self, dt: f32) {
        let game_ctx = self.game.ctx();
        let asset_manager = game_ctx.asset_manager;
        let script_manager = game_ctx.script_manager;
        let world_ecs = game_ctx.cur_world_ecs;

        let player_pos = world_ecs.get_player_position().position;

        // Update scripts here
        if let Err(e) = run_scripts(dt, world_ecs, script_manager) {
            onscreen_error!("{}", e);
        }

        // Update the camera
        self.camera_manager.update_active(
            world_ecs,
            &self.current_room,
            player_pos,
        );

        update_animation_sytem(
            world_ecs,
            asset_manager,
            dt, 
            self.current_room.id,
        ).await;
    }

    pub fn render(&mut self, alpha: f32, cur_window_size: &mut (u32, u32)) {
        clear_background(BLACK);

        // Update the render system if the window is resized
        let cur_screen = (screen_width() as u32, screen_height() as u32);
        if cur_screen != *cur_window_size {
            self.render_system.resize(cur_screen.0, cur_screen.1);
            *cur_window_size = cur_screen;
        }

        let world = &mut self.game.worlds
            .iter_mut()
            .find(|w| w.id == self.game.current_world_id)
            .expect("Current world id not present in game.");

        let interpolated_target = lerp(
            self.camera_manager.previous_position.unwrap_or_default(),
            self.camera_manager.active.camera.target,
            alpha,
        );

        // Create a new interpolated camera
        let render_cam = Camera2D {
            target: interpolated_target,
            zoom: self.camera_manager.active.camera.zoom,
            ..Default::default()
        };

        render_room(
            &world.world_ecs, 
            &self.current_room, 
            &mut self.game.asset_manager,
            &mut self.render_system,
            &render_cam,
            alpha,
            Some(&self.prev_positions),
        );

        self.render_system.present_game();
    }

    /// Updates the previous position for all entities in the active room.
    fn store_previous_positions(&mut self) {
        let current_world = self.game.current_world_mut();

        let pos_store = current_world.world_ecs.get_store::<Position>();
        let room_store = current_world.world_ecs.get_store::<CurrentRoom>();

        // Store the camera target
        self.camera_manager.previous_position = Some(self.camera_manager.active.camera.target);

        self.prev_positions = pos_store.data
            .iter()
            .filter_map(|(entity, pos)| {
                room_store.get(*entity).filter(|cr| cr.0 == self.current_room.id)
                    .map(|_| (*entity, pos.position))
            })
            .collect();
    }
}