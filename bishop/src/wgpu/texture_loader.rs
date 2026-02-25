//! Texture loading functions for the wgpu backend.

use crate::types::Texture2D;
use crate::wgpu::WgpuTexture;
use std::cell::RefCell;
use std::sync::Arc;

// TODO: Refactor to context-based texture loading (ctx.load_texture(path))
// instead of thread-local state. This would:
// - Eliminate global state
// - Work correctly with multi-threaded texture loading
// - Support multiple contexts (multiple windows, headless testing)
// - Match wgpu's natural ownership model
// Requires updating AssetManager to receive context when loading textures.

// Thread-local storage for wgpu resources needed for texture loading.
// These must be initialized by calling `init_texture_loader` before use.
thread_local! {
    static TEXTURE_RESOURCES: RefCell<Option<TextureResources>> = const { RefCell::new(None) };
}

/// Wgpu resources needed for texture creation.
struct TextureResources {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
}

/// Initializes the texture loader with wgpu resources.
/// Must be called once after creating a WgpuContext.
pub fn init_texture_loader(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
) {
    TEXTURE_RESOURCES.with(|cell| {
        *cell.borrow_mut() = Some(TextureResources {
            device,
            queue,
            bind_group_layout,
        });
    });
}

/// Loads a texture from the given path asynchronously.
pub async fn load_texture(path: &str) -> Result<Texture2D, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    TEXTURE_RESOURCES.with(|cell| {
        let resources = cell.borrow();
        let resources = resources
            .as_ref()
            .ok_or_else(|| "Texture loader not initialized. Call init_texture_loader first.".to_string())?;

        let wgpu_texture = WgpuTexture::from_png(
            &resources.device,
            &resources.queue,
            &resources.bind_group_layout,
            &data,
        ).map_err(|e| format!("Failed to decode '{}': {}", path, e))?;

        Ok(Texture2D::from_wgpu(wgpu_texture))
    })
}

/// Creates an empty 1x1 transparent texture.
pub fn empty_texture() -> Texture2D {
    TEXTURE_RESOURCES.with(|cell| {
        let resources = cell.borrow();
        let resources = resources
            .as_ref()
            .expect("Texture loader not initialized. Call init_texture_loader first.");

        let data: [u8; 4] = [0, 0, 0, 0];
        let wgpu_texture = WgpuTexture::from_rgba(
            &resources.device,
            &resources.queue,
            &resources.bind_group_layout,
            &data,
            1,
            1,
        );

        Texture2D::from_wgpu(wgpu_texture)
    })
}
