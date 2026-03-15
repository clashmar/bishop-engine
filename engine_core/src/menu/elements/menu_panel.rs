use bishop::prelude::*;
use serde::{Deserialize, Serialize};

/// Fill type for a panel background.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PanelFill {
    /// Solid color fill.
    SolidColor(Color),
}

impl Default for PanelFill {
    fn default() -> Self {
        PanelFill::SolidColor(Color::new(0.3, 0.3, 0.35, 1.0))
    }
}

/// Visual style for a decorative panel background.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PanelBackground {
    /// The fill type and color.
    pub fill: PanelFill,
    /// Opacity multiplier applied when rendering (0.0–1.0).
    pub opacity: f32,
}

impl PanelBackground {
    /// Returns the final color to use when rendering, with opacity applied.
    pub fn render_color(&self) -> Color {
        match self.fill {
            PanelFill::SolidColor(color) => {
                Color::new(color.r, color.g, color.b, color.a * self.opacity)
            }
        }
    }
}

impl Default for PanelBackground {
    fn default() -> Self {
        Self {
            fill: PanelFill::default(),
            opacity: 1.0,
        }
    }
}
