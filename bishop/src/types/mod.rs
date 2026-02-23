//! Core types for the bishop graphics abstraction.

mod color;
mod rect;

pub use color::*;
pub use rect::*;

pub use glam::{IVec2, Mat2, Mat4, Vec2, Vec3, ivec2, vec2, vec3};

/// Texture type from the active graphics backend.
#[cfg(feature = "macroquad")]
pub use macroquad::prelude::Texture2D;

/// Filter mode for texture sampling.
#[cfg(feature = "macroquad")]
pub use macroquad::texture::FilterMode;
