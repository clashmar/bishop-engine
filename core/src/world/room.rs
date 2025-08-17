use macroquad::prelude::*;
use crate::{constants::*, tilemap::TileMap};

#[derive(Clone, Debug)]
pub struct RoomMetadata {
    pub name: String,
    pub position: Vec2,
    pub size: Vec2,
    pub exits: Vec<Exit>,
    pub adjacent_rooms: Vec<usize>,
}

pub struct Room {
    pub variants: Vec<RoomVariant>,
}

impl Default for RoomMetadata {
    fn default() -> Self {
        RoomMetadata {
        name: "untitled".to_string(),
        position: DEFAULT_ROOM_POSITION,
        size: DEFAULT_ROOM_SIZE,
        exits: vec![],
        adjacent_rooms: vec![],
        }
    }
}

impl RoomMetadata {
    pub fn load_room(&self) -> Room {
        // Placeholder for real load logic

        let variant = RoomVariant {
            id: "default".to_string(),
            tilemap: TileMap::new(self.size.x as usize, self.size.y as usize),
        };    
        
        Room {
            variants: vec![variant],
        }
    }

    pub fn link_exits_slice(&mut self, other_rooms: &[&RoomMetadata]) {
        let my_size = self.size;
        let epsilon = 0.01; // tolerance for floating-point comparisons

        for exit in self.exits.iter_mut() {
            exit.target_room_id = None;

            // Local to world position (Y-flip)
            let exit_world_pos = self.position + Vec2::new(exit.position.x, my_size.y - exit.position.y - 1.0);

            'other_rooms: for (idx, other_room) in other_rooms.iter().enumerate() {
                let other_size = other_room.size;

                for other_exit in &other_room.exits {
                    // World position of the other room's exit (Y-flip)
                    let other_world_pos = other_room.position
                        + Vec2::new(other_exit.position.x, other_size.y - other_exit.position.y - 1.0);

                    let linked = match exit.direction {
                        ExitDirection::Up => {
                            other_exit.direction == ExitDirection::Down &&
                            (exit_world_pos.y - (other_world_pos.y + 1.0)).abs() < epsilon &&
                            (exit_world_pos.x - other_world_pos.x).abs() < epsilon
                        }
                        ExitDirection::Down => {
                            other_exit.direction == ExitDirection::Up &&
                            (exit_world_pos.y + 1.0 - other_world_pos.y).abs() < epsilon &&
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
        let room_size = self.size;
        self.exits.iter().map(|exit| {
            // Flip y-axis: local Y increases up, world Y increases down
            let world_pos = self.position + Vec2::new(exit.position.x, room_size.y - exit.position.y - 1.0);
            (world_pos, exit.direction)
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

impl Room {
    // pub fn size(&self) -> Vec2 {
    //     self.metadata.size
    // }

    // pub fn bounds(&self) -> (f32, f32, f32, f32) {
    //     (self.metadata.position.x, self.metadata.position.y, self.metadata.size.x, self.metadata.size.y)
    // }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitDirection {
    Up,
    Right,
    Down,
    Left
}

#[derive(Clone, Debug)]
pub struct Exit {
    pub position: Vec2,                 
    pub direction: ExitDirection,      
    pub target_room_id: Option<usize>, 
}