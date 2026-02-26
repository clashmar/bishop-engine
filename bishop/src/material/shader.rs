//! Material and shader abstractions for GPU rendering.

#[cfg(feature = "macroquad")]
pub use macroquad::prelude::{
    Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc, UniformType,
    gl_use_default_material, gl_use_material, load_material,
};

#[cfg(feature = "macroquad")]
pub use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};

#[cfg(feature = "wgpu")]
pub use crate::wgpu::{
    AmbientMaterial, FinalCompositeMaterial, FullscreenQuadRenderer, GlowMaterial,
    SceneCompositeMaterial, SpotMaterial, UndarkenedMaterial,
};

#[cfg(feature = "wgpu")]
pub use crate::wgpu::{
    AmbientUniforms, GlowData, GlowUniforms, ModelUniforms, SpotLightData, SpotUniforms,
};
