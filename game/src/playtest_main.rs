// game/src/playtest_main.rs
mod camera;
use std::{env, fs};
use engine_core::{
    assets::
        asset_manager::AssetManager
    , 
    constants::*, 
    ecs::world_ecs::WorldEcs, 
    player::PlayerOld, 
    tiles::
        tilemap::{self, TileMap}, 
        world::{
        room::{Room, RoomMetadata},
        world::GridPos,
    }
};
use macroquad::prelude::*;
use ron::de::from_str;
use crate::camera::GameCamera;

/// The complete payload the editor writes for the play‑test binary.
#[derive(serde::Deserialize)]
struct PlaytestPayload {
    room: Room,
    metadata: RoomMetadata,
    world_ecs: WorldEcs,
}

#[macroquad::main("Play‑test")]
async fn main() {
    // Load the temporary file written by the editor
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <playtest_payload.ron>", args[0]);
        std::process::exit(1);
    }
    let payload_path = &args[1];
    let payload_str = fs::read_to_string(payload_path)
        .expect("could not read the temporary play‑test file");
    let PlaytestPayload {
        room,
        metadata,
        mut world_ecs,
    } = from_str(&payload_str).expect("failed to deserialize play‑test payload");

    let mut asset_manager = AssetManager::new(&mut world_ecs).await;

    // Initialise the player exactly like the demo does.
    let start_grid = GridPos::new(4, 4);
    let start_world = tilemap::tile_to_world(start_grid);
    let mut player = PlayerOld {
        grid_position: start_grid,
        actual_position: start_world,
        velocity_x: 0.0,
        velocity_y: 0.0,
        is_airborne: false,
        has_double_jump: true,
        color: BLUE,
    };

    // Camera
    let mut cam = GameCamera {
        position: player.actual_position,
        camera: Camera2D::default(),
    };

    // Game loop
    let map: TileMap = room.variants[0].tilemap.clone();

    loop {
        // Logic
        player.update(&map);
        cam.position = player.actual_position;

        // Rendering
        cam.update_camera();


        let camera = Camera2D {
            target: vec2(cam.position.x, cam.position.y),
            zoom: vec2(2. / screen_width(), 2. / screen_height()),
            ..Default::default()
        };

        map.draw(&camera, &metadata.exits, &world_ecs, &mut asset_manager);

        println!("{}, {}", player.actual_position.x, player.actual_position.y);
        draw_rectangle(
            player.actual_position.x,
            player.actual_position.y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            player.color,
        );

        // Exit
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        next_frame().await;
    }
}