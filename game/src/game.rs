use engine_core::{
    assets::asset_manager::AssetManager, constants::*, ecs::{
        component::{CurrentRoom, Player, Position}, 
        entity::Entity
    }, storage::core_storage, world::{room::Room, world::World}
};
use crate::{modes::Mode};
use macroquad::prelude::*;
use crate::camera::GameCamera;

// #[derive(Debug, Clone)]
pub struct GameState {
    /// The whole world, including its persistent ECS.
    world: World,
    /// Cached handle to the player entity.
    player_entity: Entity,
    /// Camera that follows the player.
    camera: GameCamera,
    /// Current room.
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
            .or_else(|| world.rooms_metadata.first().map(|m| m.id))
            .expect("World has no starting room nor any room metadata");

        let current_room = core_storage::load_room(&world.id, start_room_id)
            .expect("Failed to load the starting room");

        let starting_position = world.starting_position.unwrap();

        let player_entity = world.world_ecs
            .create_entity()
            .with(Position { position: starting_position })
            .with(CurrentRoom(start_room_id))
            .with(Player)
            .finish();

        let camera = GameCamera {
            position: starting_position,
            camera: Camera2D::default(),
        };

        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        Self {
            world,
            player_entity,
            camera,
            current_room,
            mode: Mode::Explore,
            asset_manager,
        }
    }

    pub fn update(&mut self) {
        let player_position = self
            .world
            .world_ecs
            .get_store::<Position>()
            .get(self.player_entity)
            .unwrap()
            .position;

        self.camera.position = player_position;

        if is_key_pressed(KeyCode::C) {
            self.toggle_mode();
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Explore => Mode::Combat,
            Mode::Combat => Mode::Explore,
        };
    }

    pub fn draw(&mut self) {
        clear_background(BLACK);
        self.camera.update_camera();
        
        self.current_room.variants[0].tilemap.draw(
            &self.camera.camera,
            &Vec::new(),
            &self.world.world_ecs,
            &mut self.asset_manager,
        );

        draw_rectangle(
            self.camera.position.x,
            self.camera.position.y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            BLUE,
        );
    }
}