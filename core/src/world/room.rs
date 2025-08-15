use macroquad::prelude::*;
use crate::{tilemap::TileMap};

pub struct Room {
    pub name: String,
    pub position: Vec2,      
    pub variants: Vec<RoomVariant>,  
    pub exits: Vec<Exit>,
    pub adjacent_rooms: Vec<usize>,   
}

impl Room {
    pub fn size(&self) -> Vec2 {
        if let Some(first_variant) = self.variants.first() {
            Vec2::new(
                first_variant.tilemap.width as f32,
                first_variant.tilemap.height as f32,
            )
        } else {
            Vec2::new(0.0, 0.0)
        }
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let width = self.variants[0].tilemap.width as f32;
        let height = self.variants[0].tilemap.height as f32;
        (self.position.x, self.position.y, width, height)
    }

    pub fn link_exits_slice(&mut self, other_rooms: &[&Room]) {
        let my_size = self.size();
        let epsilon = 0.01; // tolerance for floating-point comparisons

        for exit in self.exits.iter_mut() {
            exit.target_room_id = None;

            // Local to world position (Y-flip)
            let exit_world_pos = self.position + Vec2::new(exit.position.x, my_size.y - exit.position.y - 1.0);

            'other_rooms: for (idx, other_room) in other_rooms.iter().enumerate() {
                let other_size = other_room.size();

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
        let room_size = self.size();
        self.exits.iter().map(|exit| {
            // Flip y-axis: local Y increases up, world Y increases down
            let world_pos = self.position + Vec2::new(exit.position.x, room_size.y - exit.position.y - 1.0);
            (world_pos, exit.direction)
        }).collect()
    }
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