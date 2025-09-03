use crate::tiles::tilemap::TileMap;
use std::{io, path::PathBuf};
use uuid::Uuid;
use serde_with::FromInto;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::{constants::*};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomMetadata {
    pub id: Uuid, 
    pub name: String,
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub size: Vec2,
    pub exits: Vec<Exit>,
    pub adjacent_rooms: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct Room {
    pub variants: Vec<RoomVariant>,
}

impl Default for RoomMetadata {
    fn default() -> Self {
        RoomMetadata {
        id: Uuid::new_v4(),
        name: "untitled".to_string(),
        position: DEFAULT_ROOM_POSITION,
        size: DEFAULT_ROOM_SIZE,
        exits: vec![],
        adjacent_rooms: vec![],
        }
    }
}

impl RoomMetadata {
    pub fn load_room(&self, world_name: &str) -> io::Result<Room> {
        let path = PathBuf::from(WORLD_SAVE_FOLDER)
            .join(world_name)
            .join("rooms")
            .join(format!("{}.ron", self.id));
        let data = std::fs::read_to_string(path)?;
        ron::de::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn link_exits(&mut self, other_rooms: &[&RoomMetadata]) {
        let epsilon = 0.01; // tolerance for floating-point comparisons

        for exit in self.exits.iter_mut() {
            exit.target_room_id = None;

            // Local to world position
            let exit_world_pos = ( self.position / TILE_SIZE ) + exit.position;

            'other_rooms: for (idx, other_room) in other_rooms.iter().enumerate() {
                for other_exit in &other_room.exits {
                    // World position of the other room's exit
                    let other_world_pos = (other_room.position / TILE_SIZE) + other_exit.position;

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
                        exit.target_room_id = Some(idx);
                        break 'other_rooms;
                    }
                }
            }
        }
    }

    pub fn world_exit_positions(&self) -> Vec<(Vec2, ExitDirection)> {
        self.exits.iter().map(|exit| {
            (self.position / TILE_SIZE + exit.position, exit.direction)
        }).collect()
    }
}



impl Default for Room {
    fn default() -> Self {
        let first_variant = RoomVariant {
            id: "default".to_string(),
            tilemap: TileMap::new(DEFAULT_ROOM_SIZE.x as usize, DEFAULT_ROOM_SIZE.y as usize),
        };

        Self { variants: vec![first_variant] }
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitDirection {
    Up,
    Right,
    Down,
    Left
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Exit {
    #[serde_as(as = "FromInto<[f32; 2]>")]
    // Local grid coordinate
    pub position: Vec2,                 
    pub direction: ExitDirection,      
    pub target_room_id: Option<usize>, 
}