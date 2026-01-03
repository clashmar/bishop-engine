// engine_core/src/camera/game_camera.rs
use crate::ecs::component::CurrentRoom;
use crate::ecs::component::Position;
use crate::ecs::entity::Entity;
use crate::world::room::RoomId;
use crate::engine_global::*;
use crate::ecs::ecs::Ecs;
use serde_with::{serde_as, FromInto};
use serde::{Deserialize, Serialize};
use ecs_component::ecs_component;
use strum_macros::EnumIter;
use macroquad::prelude::*;
use std::fmt;

#[derive(Debug, Default)]
pub struct GameCamera {
    pub camera: Camera2D,
    pub id: usize,
}

impl Clone for GameCamera {
    fn clone(&self) -> Self {
        Self {
            camera: Camera2D {
                target: self.camera.target,
                zoom: self.camera.zoom,
                rotation: self.camera.rotation,
                offset: self.camera.offset,
                render_target: self.camera.render_target.clone(),
                ..Default::default()
            },
            id: self.id,
        }
    }
}

pub fn world_virtual_width() -> f32 { cam_tile_dims().0 * tile_size() }
pub fn world_virtual_height() -> f32 { cam_tile_dims().1 * tile_size() }

/// Component for a room camera used by the game.
#[ecs_component]
#[serde_as] 
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct RoomCamera {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub zoom: Vec2,
    pub room_id: RoomId,
    pub zoom_mode: ZoomMode,
    pub camera_mode: CameraMode,
}

impl RoomCamera {
    pub fn new(room_id: RoomId) -> Self {
        let zoom = vec2(1.0 / world_virtual_width() * 2.0, 1.0 / world_virtual_height() * 2.0);
        RoomCamera { 
            zoom, 
            room_id,
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

/// Returns every `GameCamera` for a room from its id.
pub fn get_room_cameras(world_ecs: &Ecs, room_id: RoomId) -> Vec<(Entity, RoomCamera)> {
    let cam_store = world_ecs.get_store::<RoomCamera>();
    let room_store = world_ecs.get_store::<CurrentRoom>();

    cam_store
        .data
        .iter()
        .filter_map(|(entity, room_cam)| {
            let cur = room_store.get(*entity)?;
            if cur.0 != room_id {
                return None;
            }
            Some((*entity, *room_cam))
        })
        .collect()
}

/// Converts a `RoomCamera` component into a `GameCamera` from its Entity.
pub fn room_to_game_camera(
    world_ecs: &Ecs, 
    entity: &Entity, 
    room_camera: &RoomCamera,
    player_pos: Vec2, 
) -> GameCamera {
    let pos_store  = world_ecs.get_store::<Position>();

    // If the camera is a Follow cam user the player as the target
    let target = match room_camera.camera_mode {
        CameraMode::Follow(_) => player_pos,
        CameraMode::Fixed => {
            pos_store
                .data
                .get(entity)
                .expect("Camera should always have a Position component")
                .position
        }
    };

    // Build the GameCamera
    let camera = Camera2D {
        target,
        zoom: room_camera.zoom,
        render_target: Some(game_render_target()),
        ..Default::default()
    };

    GameCamera { camera, id: entity.0 }  
}

/// Returns a `GameCamera` for a room from its id, if one exists.
pub fn get_room_camera(ecs: &Ecs, room_id: RoomId) -> Option<GameCamera> {
    let pos_store = ecs.get_store::<Position>();
    let cam_store = ecs.get_store::<RoomCamera>();
    let room_store = ecs.get_store::<CurrentRoom>();

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

            return Some(GameCamera { camera, id: entity.0});
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

