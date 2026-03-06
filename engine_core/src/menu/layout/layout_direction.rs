use serde::{Deserialize, Serialize};

/// Direction for menu element layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutDirection {
    /// Stack elements vertically.
    Vertical,
    /// Stack elements horizontally.
    Horizontal,
    /// Arrange elements in a grid with specified columns.
    Grid { columns: u32 },
}

impl Default for LayoutDirection {
    fn default() -> Self {
        Self::Vertical
    }
}
