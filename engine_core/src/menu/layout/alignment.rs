use serde::{Deserialize, Serialize};

/// Horizontal alignment for menu elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

impl Default for HorizontalAlign {
    fn default() -> Self {
        Self::Center
    }
}

/// Vertical alignment for menu elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

impl Default for VerticalAlign {
    fn default() -> Self {
        Self::Middle
    }
}

/// Combined alignment for menu elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alignment {
    pub horizontal: HorizontalAlign,
    pub vertical: VerticalAlign,
}

impl Default for Alignment {
    fn default() -> Self {
        Self {
            horizontal: HorizontalAlign::Center,
            vertical: VerticalAlign::Middle,
        }
    }
}

impl Alignment {
    /// Creates an alignment with specified horizontal and vertical values.
    pub fn new(horizontal: HorizontalAlign, vertical: VerticalAlign) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Creates a centered alignment.
    pub fn center() -> Self {
        Self::default()
    }

    /// Creates a top-left alignment.
    pub fn top_left() -> Self {
        Self::new(HorizontalAlign::Left, VerticalAlign::Top)
    }

    /// Creates a top-center alignment.
    pub fn top_center() -> Self {
        Self::new(HorizontalAlign::Center, VerticalAlign::Top)
    }

    /// Creates a top-right alignment.
    pub fn top_right() -> Self {
        Self::new(HorizontalAlign::Right, VerticalAlign::Top)
    }
}
