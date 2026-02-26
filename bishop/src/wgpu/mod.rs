//! Wgpu backend for bishop.
//!
//! This module provides a wgpu-based implementation of the bishop traits,
//! using winit for window management and input handling.

mod context;
mod conversions;
mod graphics_state;
mod impls;
mod input_state;
mod render;
mod texture_loader;
mod time_state;

pub use context::WgpuContext;
pub use graphics_state::GraphicsStateError;
pub use render::{
    AmbientMaterial, AmbientUniforms, BishopRenderTarget, FinalCompositeMaterial,
    FullscreenQuadRenderer, GlowData, GlowMaterial, GlowUniforms, ModelUniforms,
    SceneCompositeMaterial, SpotLightData, SpotMaterial, SpotUniforms, UndarkenedMaterial,
    WgpuTexture, create_texture_bind_group_layout,
};
pub use texture_loader::{empty_texture, init_texture_loader, load_texture};
