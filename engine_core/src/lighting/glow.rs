// engine_core/src/lighting/glow.rs
use macroquad::prelude::*;
use reflect_derive::Reflect;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use crate::{
    assets::sprite::SpriteId, ecs_component, inspector_module
};

ecs_component!(Glow);
inspector_module!(Glow);

/// A single glow source.  
#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug, Reflect)]
#[serde(default)]
pub struct Glow {
    #[serde_as(as = "FromInto<[f32; 3]>")]
    pub color: Vec3,              
    pub intensity: f32,
    pub brightness: f32,
    pub emission: f32,
    #[widget("png")]          
    pub sprite_id: SpriteId,
}

impl Default for Glow {
    fn default() -> Self {
        Self {
            color: vec3(1.0, 1.0, 1.0),
            intensity: 0.1,
            brightness: 0.5,
            emission: 0.0,
            sprite_id: SpriteId(0),
        }
    }
}
