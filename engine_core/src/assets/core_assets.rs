// engine_core/src/assets/core_assets.rs
use std::io::Cursor;
use image::imageops::FilterType;
use image::ImageReader;
use macroquad::prelude::*;
use std::sync::LazyLock;

pub static GNF_FONT: LazyLock<Font> = LazyLock::new(|| {
    let mut font = load_ttf_font_from_bytes(include_bytes!("fonts/gnf.regular.ttf")).expect("Failed to load font.");
    font.set_filter(FilterMode::Nearest);
    let extra_chars: Vec<char> = vec!['⌘','⌥','⇧','↓','→'];
    font.populate_font_cache(&extra_chars, 15);

    font
});

/// Helper that decodes a PNG, resizes it and returns the raw RGBA bytes.
pub fn load_rgba_resized<const N: usize>(
    data: &[u8],
    size: u32,
) -> [u8; N] {
    let img = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Png)
        .decode()
        .expect("failed to decode PNG");

    let resized = img.resize_exact(size, size, FilterType::Nearest);

    let raw = resized.to_rgba8().into_raw();

    assert_eq!(raw.len(), N, "unexpected pixel count after resize");

    let mut out = [0u8; N];
    out.copy_from_slice(&raw);
    out
}
