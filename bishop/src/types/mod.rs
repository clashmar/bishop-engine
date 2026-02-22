//! Core types for the bishop graphics abstraction.

mod color;
mod rect;

pub use color::*;
pub use rect::*;

pub use glam::{IVec2, Mat2, Mat4, Vec2, Vec3, ivec2, vec2, vec3};

/// Re-export Texture2D from macroquad when using macroquad backend.
#[cfg(feature = "macroquad")]
pub use macroquad::prelude::Texture2D;

/// Re-export FilterMode from macroquad when using macroquad backend.
#[cfg(feature = "macroquad")]
pub use macroquad::texture::FilterMode;
