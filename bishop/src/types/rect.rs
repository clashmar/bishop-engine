//! Axis-aligned rectangle type.

use glam::Vec2;

/// Axis-aligned rectangle defined by position and size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// Creates a new rectangle.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Returns true if the point is inside the rectangle.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    /// Left (minimum x) edge.
    #[inline]
    pub const fn left(&self) -> f32 {
        self.x
    }

    /// Right (maximum x) edge.
    #[inline]
    pub const fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Top (minimum y) edge.
    #[inline]
    pub const fn top(&self) -> f32 {
        self.y
    }

    /// Bottom (maximum y) edge.
    #[inline]
    pub const fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Returns the top-left corner.
    pub fn top_left(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Returns the bottom-right corner.
    pub fn bottom_right(&self) -> Vec2 {
        Vec2::new(self.x + self.w, self.y + self.h)
    }

    /// Returns the size of the rect.
    pub const fn size(&self) -> Vec2 {
        Vec2::new(self.w, self.h)
    }
}
