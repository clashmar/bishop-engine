// engine_core/src/world/room.rs
use crate::{
    ecs::{
        component::{CurrentRoom, Position, RoomCamera}, 
        world_ecs::WorldEcs
    }, global::tile_size, tiles::tilemap::TileMap
};
use std::{io, path::PathBuf};
use uuid::Uuid;
use serde_with::FromInto;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::{constants::*};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Room {
    pub id: Uuid, 
    pub name: String,
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2, // Top-left origin in pixels
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub size: Vec2,
    pub exits: Vec<Exit>,
    pub adjacent_rooms: Vec<Uuid>,
    pub variants: Vec<RoomVariant>,
    pub darkness: f32,
}

impl Room {
    pub fn default(world_ecs: &mut WorldEcs) -> Self {
        let first_variant = RoomVariant {
            id: "default".to_string(),
            tilemap: TileMap::new(DEFAULT_ROOM_SIZE.x as usize, DEFAULT_ROOM_SIZE.y as usize),
        };

        let room = Room {
        id: Uuid::new_v4(),
        name: "untitled".to_string(),
        position: DEFAULT_ROOM_POSITION,
        size: DEFAULT_ROOM_SIZE,
        exits: vec![],
        adjacent_rooms: vec![],
        variants: vec![first_variant],
        darkness: 0.,
        };

        let _camera = room.create_room_camera(world_ecs);

        room
    }

    pub fn load_room(&self, world_name: &str) -> io::Result<Room> {
        let path = PathBuf::from(GAME_SAVE_ROOT)
            .join(world_name)
            .join("rooms")
            .join(format!("{}.ron", self.id));
        let data = std::fs::read_to_string(path)?;
        ron::de::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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

    pub fn create_room_camera(&self, world_ecs: &mut WorldEcs) {
        let _camera = world_ecs.create_entity()
            .with(Position { position: self.position })
            .with(RoomCamera { 
                zoom: vec2(0., 0.),
            })
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
    pub target_room_id: Option<Uuid>, 
}