// engine_core/src/lighting/light.rs
use macroquad::prelude::*;
use reflect_derive::Reflect;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use crate::{ecs_component, inspector_module};

#[serde_as]
#[derive(Clone, Copy, Serialize, Deserialize, Default, Reflect)]
pub struct Light {
    /// World position (same coordinate system as `Position`).
    #[serde_as(as = "FromInto<[f32; 2]>")]
    pub position: Vec2,
    /// Radius in world units.
    pub radius: f32,
    /// Light colour.
    #[serde_as(as = "FromInto<[f32; 3]>")]
    pub colour: Vec3,
    /// Intensity multiplier.
    pub intensity: f32,
}

ecs_component!(Light);
inspector_module!(Light);

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct LightUniform {
    // world position (x, y) + radius in the same vec3
    pub pos_radius: [f32; 3],
    // colour (r, g, b) – intensity folded into colour
    pub colour: [f32; 3],
}

impl From<Light> for LightUniform {
    fn from(l: Light) -> Self {
        LightUniform {
            pos_radius: [l.position.x, l.position.y, l.radius],
            colour: [
                l.colour.x * l.intensity,
                l.colour.y * l.intensity,
                l.colour.z * l.intensity,
            ],
        }
    }
}