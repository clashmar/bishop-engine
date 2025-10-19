use std::collections::HashMap;

// game/src/game.rs
use engine_core::{
    animation::animation_system::update_animation_sytem, assets::asset_manager::AssetManager, camera::game_camera::get_room_camera, ecs::{component::{
        CurrentRoom, Position, Velocity
    }, entity::Entity}, rendering::{render_room::render_room, render_system::RenderSystem}, storage::core_storage, world::{
        room::Room, 
        world::World
    }
};
use crate::{
    input::player_input::update_player_input, 
    physics::physics_system::update_physics
};
use macroquad::prelude::*;
use engine_core::camera::game_camera::GameCamera;

pub struct GameState {
    /// The whole world, including its persistent ecs.
    world: World,
    /// Camera that follows the player.
    camera: GameCamera,
    /// Current room
    current_room: Room,
    /// Asset Manager.
    asset_manager: AssetManager,
    /// Lighting system for the game.
    render_system: RenderSystem,
    /// Holds the Position of every entity rendered in the previous frame.
    prev_positions: HashMap<Entity, Vec2>,
}

impl GameState {
    pub async fn new() -> Self {
        let world_id = core_storage::most_recent_world_id()
            .expect("No world folder found in assets/worlds");

        let mut world = core_storage::load_world_by_id(&world_id)
            .expect("Failed to deserialize world.ron");

        let start_room_id = world
            .starting_room
            .or_else(|| world.rooms.first().map(|m| m.id))
            .expect("World has no starting room nor any rooms");

        let current_room = world
            .rooms
            .iter()
            .find(|m| m.id == start_room_id)
            .expect("Missing id for the starting room")
            .clone();

        let camera = get_room_camera(&world.world_ecs, current_room.id)
            .expect("Tested room was missing a camera.");

        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        Self {
            world,
            camera,
            current_room,
            asset_manager,
            render_system: RenderSystem::new(),
            prev_positions: HashMap::new(),
        }
    }

    pub async fn for_room(
        room: Room,
        mut world: World,
    ) -> Self {
        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        let camera = get_room_camera(&world.world_ecs, room.id)
            .expect("Tested room was missing a camera.");

        Self {
            world,
            camera,
            current_room: room,
            asset_manager,
            render_system: RenderSystem::new(),
            prev_positions: HashMap::new(),
        }
    }

    pub fn fixed_update(&mut self, dt: f32) {
        // Store the current positions for the next frame
        self.refresh_previous_positions();

        let player = self.world.world_ecs.get_player_entity();

        // If an entity exits the current room
        if let Some((exiting_entity, target_id, new_pos)) = update_physics(&mut self.world.world_ecs, &self.current_room, dt) {
            let new_room = self.world
                .rooms
                .iter()
                .find(|r| r.id == target_id)
                .expect("Target room not found");

            // Only update the new current room if the player exits
            if exiting_entity == player {
                self.current_room = new_room.clone();

                self.camera = get_room_camera(&self.world.world_ecs, new_room.id)
                    .expect("New room missing a camera");
            }

            let cur_room_mut = self.world.world_ecs.get_mut::<CurrentRoom>(exiting_entity).unwrap();
            cur_room_mut.0 = new_room.id;

            let pos_mut = self.world.world_ecs.get_mut::<Position>(exiting_entity).unwrap();
            pos_mut.position = new_pos;
        }
    }

    pub async fn update_async(&mut self, dt: f32) {
        let player = self.world.world_ecs.get_player_entity();
        let player_vel = self.world.world_ecs
            .get_store_mut::<Velocity>()
            .get_mut(player)
            .expect("Player must have a Velocity component");
        update_player_input(player_vel);

        update_animation_sytem(
            &mut self.world.world_ecs,
            &mut self.asset_manager,
            dt, 
            self.current_room.id,
        ).await;
    }

    pub fn render(&mut self, alpha: f32) {
        clear_background(BLUE);

        render_room(
            &self.world.world_ecs, 
            &self.current_room, 
            &mut self.asset_manager,
            &mut self.render_system,
            &self.camera.camera,
            alpha,
            Some(&self.prev_positions),
        );

        self.render_system.present_game();
    }

    /// Updates the previous position for all entities in the active room.
    fn refresh_previous_positions(&mut self) {
        self.prev_positions.clear();
        let pos_store = self.world.world_ecs.get_store::<Position>();
        let room_store = self.world.world_ecs.get_store::<CurrentRoom>();

        for (entity, pos) in pos_store.data.iter() {
            if let Some(cr) = room_store.get(*entity) {
                if cr.0 == self.current_room.id {
                    self.prev_positions.insert(*entity, pos.position);
                }
            }
        }
    }
}