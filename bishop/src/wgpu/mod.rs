//! Wgpu backend for bishop.
//!
//! This module provides a wgpu-based implementation of the bishop traits,
//! using winit for window management and input handling.

mod context;
mod conversions;
mod graphics_state;
mod impls;
mod input_state;
mod time_state;

pub use context::WgpuContext;
pub use graphics_state::GraphicsStateError;
pub use impls::WgpuTexture;
