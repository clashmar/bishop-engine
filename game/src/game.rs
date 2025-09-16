use engine_core::{
    assets::asset_manager::AssetManager, ecs::world_ecs::WorldEcs, rendering::render_entities::draw_entities, storage::core_storage, world::{
        room::{Room, RoomMetadata}, 
        world::World
    }
};
use crate::modes::Mode;
use macroquad::prelude::*;
use crate::camera::GameCamera;

// #[derive(Debug, Clone)]
pub struct GameState {
    /// The whole world, including its persistent ecs.
    world: World,
    /// Camera that follows the player.
    camera: GameCamera,
    /// Current room.
    current_room: Room,
    /// Current room metadat
    current_room_metadata: RoomMetadata,
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

        let current_room_metadata = world
            .rooms_metadata
            .iter()
            .find(|m| m.id == start_room_id)
            .expect("Missing metadata for the starting room")
            .clone();

        let starting_position = world.starting_position.unwrap();

        let camera = GameCamera {
            position: starting_position,
            camera: Camera2D::default(),
        };

        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        Self {
            world,
            camera,
            current_room,
            current_room_metadata,
            mode: Mode::Explore,
            asset_manager,
        }
    }

    pub async fn for_room(
        current_room: Room,
        current_room_metadata: RoomMetadata,
        mut world: World,
    ) -> Self {
        let asset_manager = AssetManager::new(&mut world.world_ecs).await;

        // TODO: GIVE ROOM A CAMERA AND USE THAT
        let starting_position = current_room_metadata.position;

        let camera = GameCamera {
            position: starting_position,
            camera: Camera2D::default(),
        };

        Self {
            world,
            camera,
            current_room,
            current_room_metadata,
            mode: Mode::Explore,
            asset_manager,
        }
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::C) {
            self.toggle_mode();
        }
    }

    pub fn draw(&mut self) {
        clear_background(BLACK);
        self.camera.update_camera();
        
        self.current_room.variants[0].tilemap.draw(
            &self.camera.camera,
            &self.current_room_metadata.exits,
            &self.world.world_ecs,
            &mut self.asset_manager,
        );

        draw_entities(
            &self.world.world_ecs, 
            &self.current_room_metadata, 
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