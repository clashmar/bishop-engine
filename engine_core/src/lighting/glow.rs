// engine_core/src/lighting/glow.rs
use macroquad::prelude::*;
use reflect_derive::Reflect;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use crate::{
    assets::asset_manager::AssetManager, 
    ecs::{
        component::PostCreate, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, 
    ecs_component, inspector_module
};

ecs_component!(Glow);
inspector_module!(Glow);

/// A single glow source.  
#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug, Reflect)]
pub struct Glow {
    #[serde_as(as = "FromInto<[f32; 3]>")]
    pub color: Vec3,              
    pub intensity: f32,
    pub brightness: f32,
    #[serde_as(as = "FromInto<[f32; 2]>")]           
    pub mask_size: Vec2, // TODO get rid of this and read from sprite
    pub emission: f32,
    #[widget("png")]          
    pub sprite: String,
}

impl Default for Glow {
    fn default() -> Self {
        Self {
            color: vec3(1.0, 1.0, 1.0),
            intensity: 0.5,
            brightness: 0.0,
            mask_size: vec2(64.0, 64.0),
            emission: 2.0,
            sprite: String::new(),
        }
    }
}

impl PostCreate for Glow {
    fn post_create(
        &mut self,
        world_ecs: &mut WorldEcs,
        entity: Entity,
        asset_manager: &mut AssetManager,
    ) {
        // TODO check if the entity already has a sprite and use that when there is no path
    }
}
