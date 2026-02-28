//! Material and shader abstractions for GPU rendering.

pub use crate::wgpu::{
    AmbientMaterial, FinalCompositeMaterial, FullscreenQuadRenderer, GlowMaterial,
    SceneCompositeMaterial, SpotMaterial, UndarkenedMaterial,
};

pub use crate::wgpu::{
    AmbientUniforms, GlowData, GlowUniforms, ModelUniforms, SpotLightData, SpotUniforms,
};
