//! Font loading and management for bishop.

#[cfg(feature = "macroquad")]
mod macroquad_font {
    use macroquad::prelude as mq;
    use std::sync::LazyLock;

    static GNF_FONT: LazyLock<mq::Font> = LazyLock::new(|| {
        let mut font = mq::load_ttf_font_from_bytes(include_bytes!("fonts/gnf.regular.ttf"))
            .expect("Failed to load GNF font");
        font.set_filter(mq::FilterMode::Nearest);
        let extra_chars: Vec<char> = vec!['⌘', '⌥', '⇧', '↓', '→'];
        font.populate_font_cache(&extra_chars, 15);
        font
    });

    /// Pre-caches the GNF font with common character sizes.
    pub fn precache() {
        let font = &*GNF_FONT;
        let chars: Vec<char> = (32u8..=126).map(|c| c as char).collect();

        for size in [12, 14, 15, 16, 18, 20, 24, 28, 32, 36, 48] {
            font.populate_font_cache(&chars, size);
        }
    }

    /// Returns a clone of the GNF font.
    pub fn get_font() -> Option<mq::Font> {
        Some(GNF_FONT.clone())
    }
}

#[cfg(feature = "macroquad")]
pub use macroquad_font::*;
