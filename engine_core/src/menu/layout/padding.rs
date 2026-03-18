use serde::{Deserialize, Serialize};

/// Padding around menu elements.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Padding {
    fn default() -> Self {
        Self::uniform(0.0)
    }
}

impl Padding {
    /// Creates padding with the same value on all sides.
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Creates padding with separate horizontal and vertical values.
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Creates padding with all sides specified.
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Returns the total horizontal padding (left + right).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Returns the total vertical padding (top + bottom).
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}
