// game/src/game.rs
use std::collections::HashMap;
use engine_core::{
    animation::animation_system::update_animation_sytem, camera::camera_manager::CameraManager, ecs::{component::{
        CurrentRoom, Position, Velocity
    }, entity::Entity}, game::game::Game, rendering::{render_room::{lerp, render_room}, render_system::RenderSystem}, storage::core_storage, world::{
        room::Room, transition_manager::TransitionManager
    }
};
use crate::{
    input::player_input::update_player_input, 
    physics::physics_system::update_physics
};
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
        let game_folder = core_storage::most_recent_game_folder()
            .expect("No valid game folder found in games/");

        let game = core_storage::load_game_from_folder(&game_folder).await
            .expect("Failed to deserialize game.ron");

        let start_room_id = game.current_world().starting_room
            .or_else(|| game.worlds.first().map(|m| m.starting_room.expect("Game has no starting room.")))
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

    pub fn fixed_update(&mut self, dt: f32) {
        // Store the current positions for the next frame
        self.store_previous_positions();

        let current_world = self.game.current_world_mut();

        let player = current_world.world_ecs.get_player_entity();

        // If an entity exits the current room
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
        let world = &mut self.game.worlds
            .iter_mut()
            .find(|w| w.id == self.game.current_world_id)
            .expect("Current world id not present in game.");

        let player = world.world_ecs.get_player_entity();
        let player_pos = world.world_ecs.get_player_position().position;

        let player_vel = world.world_ecs
            .get_store_mut::<Velocity>()
            .get_mut(player)
            .expect("Player must have a Velocity component");

        update_player_input(player_vel);

        // Update the camera
        self.camera_manager.update_active(
            &world.world_ecs,
            &self.current_room,
            player_pos,
        );

        update_animation_sytem(
            &mut world.world_ecs,
            &mut self.game.asset_manager,
            dt, 
            self.current_room.id,
        ).await;
    }

    pub fn render(&mut self, alpha: f32) {
        clear_background(BLUE);

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