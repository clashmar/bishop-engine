//! Backend texture loading functions for macroquad.

use crate::types::Texture2D;
use macroquad::prelude as mq;

/// Loads a texture from the given path asynchronously.
pub async fn load_texture(path: &str) -> Result<Texture2D, String> {
    mq::load_texture(path)
        .await
        .map_err(|e| format!("Failed to load '{}': {}", path, e))
}

/// Creates an empty texture.
pub fn empty_texture() -> Texture2D {
    Texture2D::empty()
}
