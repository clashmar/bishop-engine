//! Core types for the bishop graphics abstraction.

mod color;
mod rect;

pub use color::*;
pub use rect::*;

pub use glam::{IVec2, Mat2, Mat4, Vec2, Vec3, ivec2, vec2, vec3};

/// Filter mode for texture sampling.
/// Note: This is a legacy type from macroquad. The wgpu backend hardcodes
/// Nearest filtering for pixel art and ignores this setting.
#[cfg(all(feature = "macroquad", not(feature = "wgpu")))]
pub use macroquad::texture::FilterMode;

/// Texture type from the macroquad backend.
/// Re-exported directly for backward compatibility with existing code.
#[cfg(all(feature = "macroquad", not(feature = "wgpu")))]
pub use macroquad::prelude::Texture2D;

/// Texture wrapper for wgpu backend.
#[cfg(feature = "wgpu")]
#[derive(Clone)]
pub struct Texture2D(std::sync::Arc<crate::wgpu::WgpuTexture>);

#[cfg(feature = "wgpu")]
impl Texture2D {
    /// Creates a new Texture2D from a WgpuTexture.
    pub fn from_wgpu(texture: crate::wgpu::WgpuTexture) -> Self {
        Self(std::sync::Arc::new(texture))
    }

    /// Creates a new Texture2D from an Arc-wrapped WgpuTexture.
    pub fn from_wgpu_arc(texture: std::sync::Arc<crate::wgpu::WgpuTexture>) -> Self {
        Self(texture)
    }

    /// Returns a reference to the underlying WgpuTexture.
    pub fn inner(&self) -> &crate::wgpu::WgpuTexture {
        &self.0
    }

    /// Returns the Arc-wrapped WgpuTexture.
    pub fn inner_arc(&self) -> &std::sync::Arc<crate::wgpu::WgpuTexture> {
        &self.0
    }

    /// Returns the texture width in pixels.
    pub fn width(&self) -> f32 {
        self.0.width() as f32
    }

    /// Returns the texture height in pixels.
    pub fn height(&self) -> f32 {
        self.0.height() as f32
    }

    /// No-op on wgpu. Wgpu hardcodes Nearest filtering for pixel art.
    pub fn set_filter(&self, _filter: FilterMode) {
        // Intentionally empty - wgpu uses Nearest filtering by default
    }
}

/// Filter mode for texture sampling (wgpu version).
/// Note: This is a legacy type kept for API compatibility. The wgpu backend
/// hardcodes Nearest filtering for pixel art and ignores this setting.
#[cfg(feature = "wgpu")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Linear interpolation (not used in pixel art engine).
    Linear,
    /// Nearest neighbor filtering (default for pixel art).
    Nearest,
}
