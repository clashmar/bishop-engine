// engine_core/src/camera/game_camera.rs
use crate::{ecs::{component::{CurrentRoom, Position}, world_ecs::WorldEcs}, ecs_component, global::*};
use std::fmt;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Debug)]
pub struct GameCamera {
    pub position: Vec2,
    pub camera: Camera2D,
}

pub fn world_virtual_width() -> f32 { cam_tile_dims().0 * tile_size() }
pub fn world_virtual_height() -> f32 { cam_tile_dims().1 * tile_size() }

/// Component for a room camera used by the game.
#[serde_as] 
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct RoomCamera {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub zoom: Vec2,
    pub zoom_mode: ZoomMode,
    pub camera_mode: CameraMode,
}
ecs_component!(RoomCamera);

impl Default for RoomCamera {
    fn default() -> Self {
        let zoom = vec2(1.0 / world_virtual_width() * 2.0, 1.0 / world_virtual_height() * 2.0);
        RoomCamera { 
            zoom, 
            zoom_mode: ZoomMode::Step,
            camera_mode: CameraMode::Fixed,
        }
    }
}

/// The two display modes the inspector can use.
#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum ZoomMode {
    #[default]
    Step,
    Free,
}

impl ZoomMode {
    pub fn ui_label(&self) -> String {
        match self {
            &ZoomMode::Step => "Step".to_string(),
            &ZoomMode::Free => "Free".to_string(),
        }
    }
}

impl fmt::Display for ZoomMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ui_label())
    }
}

/// The two display modes the inspector can use.
#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum CameraMode {
    #[default]
    Fixed,
    /// The camera is set to follow the player, with optional restrictions.
    Follow(FollowRestriction),
}

impl CameraMode {
    pub fn ui_label(&self) -> String {
        match self {
            &CameraMode::Fixed => "Fixed".to_string(),
            CameraMode::Follow(restriction) => format!("Follow ({})", restriction),
        }
    }
}

impl fmt::Display for CameraMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ui_label())
    }
}

/// The possible restrictions for Follow mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FollowRestriction {
    /// Camera can move freely in all directions.
    #[default]
    Free,
    /// The camera is clamped vertically.
    ClampY,
    /// The camera is clamped horizontally.
    ClampX,
}

impl fmt::Display for FollowRestriction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let txt = match self {
            FollowRestriction::Free => "Free",
            FollowRestriction::ClampY => "Clamp Y",
            FollowRestriction::ClampX => "Clamp X",
        };
        write!(f, "{}", txt)
    }
}

pub fn game_render_target() -> RenderTarget {
    let width = world_virtual_width() as u32;
    let height = world_virtual_height() as u32;
    
    let rt = render_target(
        width,
        height,
    );
    // Always use Nearest
    rt.texture.set_filter(FilterMode::Nearest);
    rt
}

/// Returns a `GameCamera` for a room from its id, if one exists.
pub fn get_room_camera(world_ecs: &WorldEcs, room_id: Uuid) -> Option<GameCamera> {
    let pos_store = world_ecs.get_store::<Position>();
    let cam_store = world_ecs.get_store::<RoomCamera>();
    let room_store = world_ecs.get_store::<CurrentRoom>();

    for (entity, room_cam) in cam_store.data.iter() {
        if let Some(current_room) = room_store.get(*entity) {
            if current_room.0 != room_id { continue; }

            let position = pos_store.data
                .get(entity)
                .expect("Camera should always have position.")
                .position;

            let camera = Camera2D {
                target: position,
                zoom: room_cam.zoom,
                render_target: Some(game_render_target()),
                ..Default::default()
            };

            return Some(GameCamera { position, camera, });
        }
    }
    None
}

pub fn zoom_from_scalar(scalar: f32) -> Vec2 {
    // Fixed virtual aspect
    let aspect = world_virtual_width() / world_virtual_height();

    if aspect >= 1.0 {
        vec2(scalar / aspect, scalar)
    } else {
        vec2(scalar, scalar * aspect)
    }
}

impl GameCamera {
    pub fn update_camera(&mut self) {
        // let cam_x = self.position.x as f32 + TILE_SIZE / 2.0;

        // // Offset the camera upwards
        // let vertical_offset = screen_height() / 2.0;
        // let cam_y = self.position.y + TILE_SIZE / 2.0 - vertical_offset;

        // self.camera.target = vec2(cam_x, cam_y);
        // self.camera.zoom = vec2(1.2 / screen_width(), 1.2 / screen_height());

        // set_camera(&self.camera);

    }

    pub fn move_camera(&mut self) {
        // let speed = 4.0; // pixels per frame
        // let input = input::get_omni_input(); // returns Vec2 (e.g. (1, 0))
        // self.position += input * speed;
    }
}

