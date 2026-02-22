//! Render target abstractions for off-screen rendering.

#[cfg(feature = "macroquad")]
pub use macroquad::texture::{FilterMode, RenderTarget, render_target};
