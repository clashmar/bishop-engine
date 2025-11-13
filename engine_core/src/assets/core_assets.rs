// engine_core/src/assets/core_assets.rs
use macroquad::prelude::*;
use std::sync::LazyLock;

pub static GNF_FONT: LazyLock<Font> = LazyLock::new(|| {
    let mut font = load_ttf_font_from_bytes(include_bytes!("fonts/gnf.regular.ttf")).expect("Failed to load font.");
    font.set_filter(FilterMode::Nearest);
    let extra_chars: Vec<char> = vec!['⌘','⌥','⇧'];
    font.populate_font_cache(&extra_chars, 15);

    font
});
