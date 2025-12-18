pub mod animation;
pub mod assets;
pub mod camera;
pub mod constants;
pub mod controls;
pub mod ecs;
pub mod engine_global;
pub mod game;
pub mod input;
pub mod lighting;
pub mod logging;
pub mod physics;
pub mod rendering;
pub mod scripting;
pub mod shaders;
pub mod storage;
pub mod tiles;
pub mod world;
pub mod ui;

/// Demo test.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}