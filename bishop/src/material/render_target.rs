//! Render target abstractions for off-screen rendering.

// FilterMode is exported from types module to avoid conflicts between backends.
#[cfg(feature = "macroquad")]
pub use macroquad::texture::{RenderTarget, render_target};

#[cfg(feature = "wgpu")]
pub use crate::wgpu::{BishopRenderTarget, create_texture_bind_group_layout};
