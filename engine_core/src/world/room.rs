// engine_core/src/world/room.rs
use crate::camera::game_camera::RoomCamera;
use crate::tiles::tilemap::TileMap;
use crate::ecs::entity::Entity;
use crate::ecs::component::*;
use crate::ecs::transform::*;
use crate::ecs::ecs::Ecs;
use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use macroquad::prelude::*;
use serde_with::FromInto;
use serde_with::serde_as;

/// Identifier for a room, globally unique across all worlds.
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
    /// Creates a default room with the given pre-allocated room ID.
    pub fn default(ecs: &mut Ecs, room_id: RoomId, grid_size: f32) -> Self {
        let first_variant = RoomVariant {
            id: "default".to_string(),
            tilemap: TileMap::new(DEFAULT_ROOM_SIZE.x as usize, DEFAULT_ROOM_SIZE.y as usize),
        };

        let room = Room {
            id: room_id,
            name: "untitled".to_string(),
            position: DEFAULT_ROOM_POSITION,
            size: DEFAULT_ROOM_SIZE,
            exits: vec![],
            adjacent_rooms: vec![],
            variants: vec![first_variant],
            darkness: 0.,
        };

        let _camera = room.create_room_camera(ecs, room_id, grid_size);

        room
    }

    /// Link exits to adjacent rooms based on their positions.
    pub fn link_exits(&mut self, other_rooms: &[&Room], grid_size: f32) {
        let epsilon = 0.01; // tolerance for floating-point comparisons

        for exit in self.exits.iter_mut() {
            exit.target_room_id = None;

            // Local to world position
            let exit_world_pos = (self.position / grid_size) + exit.position;

            'other_rooms: for (_, other_room) in other_rooms.iter().enumerate() {
                for other_exit in &other_room.exits {
                    // World position of the other room's exit
                    let other_world_pos = (other_room.position / grid_size) + other_exit.position;

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

    /// Returns the world exit positions for this room.
    pub fn world_exit_positions(&self, grid_size: f32) -> Vec<(Vec2, ExitDirection)> {
        self.exits.iter().map(|exit| {
            (self.position / grid_size + exit.position, exit.direction)
        }).collect()
    }

    pub fn create_room_camera(&self, ecs: &mut Ecs, room_id: RoomId, grid_size: f32) {
        const CAMERA_PREFIX: &str = "Camera ";
        let name_store = ecs.get_store::<Name>();
        let cur_room_store = ecs.get_store::<CurrentRoom>();

        let mut used: HashSet<usize> = HashSet::new();

        for (entity, name) in name_store.data.iter() {
            if let Some(cur_room) = cur_room_store.get(*entity) {
                if cur_room.0 != self.id {
                    continue;
                }
                if let Some(num_str) = name.strip_prefix(CAMERA_PREFIX) {
                    if let Ok(num) = num_str.parse::<usize>() {
                        if num > 0 {
                            used.insert(num);
                        }
                    }
                }
            }
        }

        let mut next_idx = 1;
        while used.contains(&next_idx) {
            next_idx += 1;
        }

        ecs.create_entity()
            .with(Transform { position: self.position, pivot: Pivot::TopLeft })
            .with(RoomCamera::new(room_id, grid_size))
            .with(CurrentRoom(self.id))
            .with(Name(format!("{}{}", CAMERA_PREFIX, next_idx)));
    }

    /// Returns the axis‑aligned rectangle that a room occupies in world space.
    #[inline]
    pub fn room_bounds(&self, grid_size: f32) -> (Vec2, Vec2) {
        let min = self.position;
        let max = self.position + self.size * grid_size;
        (min, max)
    }

    // Returns a reference to the current variant of the room.
    pub fn current_variant(&self) -> &RoomVariant {
        &self.variants[0]
    }
}

/// Returns a HashSet of all entities in the current room.
pub fn entities_in_room(ecs: &mut Ecs, room_id: RoomId) -> HashSet<Entity> {
    let room_store = ecs.get_store::<CurrentRoom>();
    room_store
        .data
        .iter()
        .filter_map(|(entity, cur_room)| {
            if cur_room.0 == room_id {
                Some(*entity)
            } else {
                None
            }
        })
        .collect()
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