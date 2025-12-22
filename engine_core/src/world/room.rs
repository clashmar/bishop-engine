// engine_core/src/world/room.rs
use crate::camera::game_camera::RoomCamera;
use crate::engine_global::tile_size;
use crate::ecs::world_ecs::WorldEcs;
use crate::tiles::tilemap::TileMap;
use crate::ecs::component::*;
use serde_with::FromInto;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::{constants::*};

/// Identifier for a room.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct RoomId(pub usize);

impl std::ops::Deref for RoomId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RoomId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Room {
    pub id: RoomId, 
    pub name: String,
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2, // Top-left origin in pixels
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub size: Vec2,
    pub exits: Vec<Exit>,
    pub adjacent_rooms: Vec<RoomId>,
    pub variants: Vec<RoomVariant>,
    pub darkness: f32,
}

impl Room {
    pub fn default(world_ecs: &mut WorldEcs) -> Self {
        let first_variant = RoomVariant {
            id: "default".to_string(),
            tilemap: TileMap::new(DEFAULT_ROOM_SIZE.x as usize, DEFAULT_ROOM_SIZE.y as usize),
        };

        let id = RoomId(0);

        let room = Room {
        id,
        name: "untitled".to_string(),
        position: DEFAULT_ROOM_POSITION,
        size: DEFAULT_ROOM_SIZE,
        exits: vec![],
        adjacent_rooms: vec![],
        variants: vec![first_variant],
        darkness: 0.,
        };

        let _camera = room.create_room_camera(world_ecs, id);

        room
    }

    pub fn link_exits(&mut self, other_rooms: &[&Room]) {
        let epsilon = 0.01; // tolerance for floating-point comparisons

        for exit in self.exits.iter_mut() {
            exit.target_room_id = None;

            // Local to world position
            let exit_world_pos = ( self.position / tile_size() ) + exit.position;

            'other_rooms: for (_, other_room) in other_rooms.iter().enumerate() {
                for other_exit in &other_room.exits {
                    // World position of the other room's exit
                    let other_world_pos = (other_room.position / tile_size()) + other_exit.position;

                    let linked = match exit.direction {
                        ExitDirection::Up => {
                            other_exit.direction == ExitDirection::Down &&
                            (exit_world_pos.y - (other_world_pos.y - 1.0)).abs() < epsilon &&
                            (exit_world_pos.x - other_world_pos.x).abs() < epsilon
                        }
                        ExitDirection::Down => {
                            other_exit.direction == ExitDirection::Up &&
                            (exit_world_pos.y - 1.0 - other_world_pos.y).abs() < epsilon &&
                            (exit_world_pos.x - other_world_pos.x).abs() < epsilon
                        }
                        ExitDirection::Left => {
                            other_exit.direction == ExitDirection::Right &&
                            (exit_world_pos.x - other_world_pos.x + 1.0).abs() < epsilon && 
                            (exit_world_pos.y - other_world_pos.y).abs() < epsilon    
                        }
                        ExitDirection::Right => {
                            other_exit.direction == ExitDirection::Left &&
                            (exit_world_pos.x - other_world_pos.x - 1.0).abs() < epsilon && 
                            (exit_world_pos.y - other_world_pos.y).abs() < epsilon
                        }
                    };

                    if linked {
                        exit.target_room_id = Some(other_room.id);
                        break 'other_rooms;
                    }
                }
            }
        }
    }

    pub fn world_exit_positions(&self) -> Vec<(Vec2, ExitDirection)> {
        self.exits.iter().map(|exit| {
            (self.position / tile_size() + exit.position, exit.direction)
        }).collect()
    }

    pub fn create_room_camera(&self, world_ecs: &mut WorldEcs, room_id: RoomId) {
        let _camera = world_ecs.create_entity()
            .with(Position { position: self.position })
            .with(RoomCamera::new(room_id))
            .with(CurrentRoom(self.id));
    }

    /// Returns the axis‑aligned rectangle that a room occupies in world space.
    #[inline]
    pub fn room_bounds(&self) -> (Vec2, Vec2) {
        let min = self.position;
        let max = self.position + self.size * tile_size();
        (min, max)
    }

    // Returns a reference to the current variant of the room.
    pub fn current_variant(&self) -> &RoomVariant {
        &self.variants[0]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RoomVariant {
    pub id: String,
    pub tilemap: TileMap,      
}

impl Default for RoomVariant {
    fn default() -> Self {
        Self {
            id: String::new(),
            tilemap: TileMap::new(10, 10),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExitDirection {
    #[default]
    Up,
    Right,
    Down,
    Left
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
#[serde(default)]
pub struct Exit {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    // Local grid coordinate
    pub position: Vec2,                 
    pub direction: ExitDirection,      
    pub target_room_id: Option<RoomId>, 
}