// engine_core/src/assets/core_assets.rs
use image::imageops::FilterType;
use macroquad::prelude::*;
use std::sync::LazyLock;
use image::ImageReader;
use std::io::Cursor;

pub static GNF_FONT: LazyLock<Font> = LazyLock::new(|| {
    let mut font = load_ttf_font_from_bytes(include_bytes!("fonts/gnf.regular.ttf")).expect("Failed to load font.");
    font.set_filter(FilterMode::Nearest);
    let extra_chars: Vec<char> = vec!['⌘','⌥','⇧','↓','→'];
    font.populate_font_cache(&extra_chars, 15);

    font
});

/// Pre-caches the font atlas with common characters and sizes.
/// Call this early at startup to avoid font rendering corruption.
/// See: https://github.com/not-fl3/macroquad/issues/1006
pub fn precache_font() {
    // Force lazy initialization
    let font = &*GNF_FONT;

    // All printable ASCII characters (32-126)
    let chars: Vec<char> = (32u8..=126).map(|c| c as char).collect();

    // Common font sizes used in the editor/game UI
    for size in [12, 14, 15, 16, 18, 20, 24, 28, 32, 36, 48] {
        font.populate_font_cache(&chars, size);
    }
}

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
