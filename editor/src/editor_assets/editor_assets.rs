// editor/src/editor_assets/editor_assets.rs
use std::io::Cursor;
use std::hash::Hasher;
use std::hash::BuildHasherDefault;
use std::hash::DefaultHasher;
use futures::executor::block_on;
use image::ImageReader;
use image::imageops::FilterType;
use std::{env, fs};
use std::hash::BuildHasher;
use std::path::PathBuf;
use std::sync::LazyLock;
use macroquad::prelude::*;

pub static ICON_SMALL: LazyLock<[u8; 16 * 16 * 4]> = LazyLock::new(|| {
    load_rgba_resized::<{ 16 * 16 * 4 }>(include_bytes!("icon.png"), 16)
});

pub static ICON_MEDIUM: LazyLock<[u8; 32 * 32 * 4]> = LazyLock::new(|| {
    load_rgba_resized::<{ 32 * 32 * 4 }>(include_bytes!("icon.png"), 32)
});

pub static ICON_BIG: LazyLock<[u8; 64 * 64 * 4]> = LazyLock::new(|| {
    load_rgba_resized::<{ 64 * 64 * 4 }>(include_bytes!("icon.png"), 64)
});

pub static SELECT_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/select.png"))
});

pub static EDIT_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/edit.png"))
});

pub static CREATE_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/create.png"))
});

pub static DELETE_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/delete.png"))
});

pub static MOVE_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/move.png"))
});

pub static TILE_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/tile.png"))
});

pub static ENTITY_ICON: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("icons/entity.png"))
});

pub static CIRCLE_120PX: LazyLock<Texture2D> = LazyLock::new(|| {
    load_texture_from_bytes(include_bytes!("textures/circle120px.png"))
});

/// Helper that turns the embedded PNG data into a `Texture2D`.
fn load_texture_from_bytes(data: &'static [u8]) -> Texture2D {
    let mut tmp_path: PathBuf = env::temp_dir();
    let hash = {
        type FnvHasher = DefaultHasher;
        let mut hasher = BuildHasherDefault::<FnvHasher>::default().build_hasher();
        hasher.write(data);
        hasher.finish()
    };

    tmp_path.push(format!("asset_{:x}.png", hash));

    if !tmp_path.exists() {
        fs::write(&tmp_path, data)
            .expect("Failed to write temporary texture file.");
    }

    let texture = block_on(async {
        load_texture(tmp_path.to_string_lossy().as_ref())
            .await
            .expect("Failed to load texture from temporary file.")
    });

    texture.set_filter(FilterMode::Nearest);
    texture
}


/// Helper that decodes a PNG, resizes it and returns the raw RGBA bytes.
fn load_rgba_resized<const N: usize>(
    data: &'static [u8],
    size: u32,
) -> [u8; N] {
    let img = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Png)
        .decode()
        .expect("failed to decode PNG");

    let resized = img.resize_exact(size, size, FilterType::Lanczos3);

    let raw = resized.to_rgba8().into_raw();

    assert_eq!(raw.len(), N, "unexpected pixel count after resize");

    let mut out = [0u8; N];
    out.copy_from_slice(&raw);
    out
}

