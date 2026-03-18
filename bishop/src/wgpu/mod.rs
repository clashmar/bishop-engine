//! Wgpu backend for bishop.
//!
//! This module provides a wgpu-based implementation of the bishop traits,
//! using winit for window management and input handling.

pub(crate) mod app_runner;
mod context;
mod conversions;
mod exec;
mod impls;
mod render;
mod state;
mod texture_loader;

pub use context::WgpuContext;
pub use exec::FrameFuture;
pub use state::GraphicsStateError;
pub use render::{
    AmbientMaterial, AmbientUniforms, BishopRenderTarget, FinalCompositeMaterial,
    FullscreenQuadRenderer, GlowData, GlowMaterial, GlowUniforms, GridMaterial, GridUniforms,
    ModelUniforms, SceneCompositeMaterial, SpotLightData, SpotMaterial, SpotUniforms,
    UndarkenedMaterial, WgpuTexture, create_texture_bind_group_layout,
};
pub use texture_loader::{empty_texture, init_texture_loader, load_texture};
