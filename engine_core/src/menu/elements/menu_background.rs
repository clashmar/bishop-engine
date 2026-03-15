use bishop::prelude::*;
use serde::{Deserialize, Serialize};

/// Background style for a menu.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MenuBackground {
    /// No background.
    None,
    /// Solid color background.
    SolidColor(Color),
    /// Semi-transparent dimming overlay.
    Dimmed(f32),
}

impl MenuBackground {
    /// Returns true if this background fully obscures the game behind the menu.
    pub fn is_opaque(&self) -> bool {
        matches!(self, MenuBackground::SolidColor(_))
    }
}

impl Default for MenuBackground {
    fn default() -> Self {
        MenuBackground::Dimmed(0.7)
    }
}
