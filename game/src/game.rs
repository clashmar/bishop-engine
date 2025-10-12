// game/src/game.rs
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::component::{
        CurrentRoom, 
        Position, 
        Velocity
    }, 
    rendering::render_room::render_entities, 
    storage::core_storage,
    world::{
        room::Room, 
        world::World
    }
};
use crate::{
    input::player_input::update_player_input, 
    modes::Mode, 
    physics::physics_system::update_physics
};
use macroquad::prelude::*;
use engine_core::camera::game_camera::GameCamera;

// #[derive(Debug, Clone)]
pub struct GameState {
    /// The whole world, including its persistent ecs.
    world: World,
    /// Camera that follows the player.
    camera: GameCamera,
    /// Current room
    current_room: Room,
    /// Current mode.
    mode: Mode,
    /// Asset Manager.
    asset_manager: AssetManager,
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

        let camera = Room::get_room_camera(&world.world_ecs, current_room.id)
            .expect("Tested room was missing a camera.");

        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        Self {
            world,
            camera,
            current_room,
            mode: Mode::Explore,
            asset_manager,
        }
    }

    pub async fn for_room(
        room: Room,
        mut world: World,
    ) -> Self {
        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        let camera = Room::get_room_camera(&world.world_ecs, room.id)
            .expect("Tested room was missing a camera.");

        Self {
            world,
            camera,
            current_room: room,
            mode: Mode::Explore,
            asset_manager,
        }
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::C) {
            self.toggle_mode();
        }

        let player = self.world.world_ecs.get_player_entity();

        let player_vel = self.world.world_ecs
            .get_store_mut::<Velocity>()
            .get_mut(player)
            .expect("Player must have a Velocity component");

        update_player_input(player_vel);

        // If an entity exits the current room
        if let Some((exiting_entity, target_id, new_pos)) = update_physics(&mut self.world.world_ecs, &self.current_room) {
            let new_room = self.world
                .rooms
                .iter()
                .find(|r| r.id == target_id)
                .expect("Target room not found");

            // Only update the new current room if the player exits
            if exiting_entity == player {
                self.current_room = new_room.clone();

                self.camera = Room::get_room_camera(&self.world.world_ecs, new_room.id)
                    .expect("New room missing a camera");
            }

            let cur_room_mut = self.world.world_ecs.get_mut::<CurrentRoom>(exiting_entity).unwrap();
            cur_room_mut.0 = new_room.id;

            let pos_mut = self.world.world_ecs.get_mut::<Position>(exiting_entity).unwrap();
            pos_mut.position = new_pos;
        }
    }

    pub fn draw(&mut self) {
        clear_background(BLUE);
        
        self.current_room.variants[0].tilemap.draw(
            &self.camera.camera,
            &self.current_room.exits,
            &self.world.world_ecs,
            &mut self.asset_manager,
            self.current_room.position,
        );

        render_entities(
            &self.world.world_ecs, 
            &self.current_room, 
            &mut self.asset_manager
        );
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Explore => Mode::Combat,
            Mode::Combat => Mode::Explore,
        };
    }
}