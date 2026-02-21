pub use glam::{IVec2, Mat2, Mat4, Vec2, Vec3, ivec2, vec2, vec3};

/// RGBA color with components in 0.0-1.0 range.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Creates a new color from RGBA components.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from RGB components with full opacity.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const LIGHTGREY: Color = Color::new(0.78, 0.78, 0.78, 1.00);
    pub const GREY: Color = Color::new(0.51, 0.51, 0.51, 1.00);
    pub const DARKGRAY: Color = Color::new(0.31, 0.31, 0.31, 1.00);
    pub const YELLOW: Color = Color::new(0.99, 0.98, 0.00, 1.00);
    pub const GOLD: Color = Color::new(1.00, 0.80, 0.00, 1.00);
    pub const ORANGE: Color = Color::new(1.00, 0.63, 0.00, 1.00);
    pub const PINK: Color = Color::new(1.00, 0.43, 0.76, 1.00);
    pub const RED: Color = Color::new(0.90, 0.16, 0.22, 1.00);
    pub const MAROON: Color = Color::new(0.75, 0.13, 0.22, 1.00);
    pub const GREEN: Color = Color::new(0.00, 0.89, 0.19, 1.00);
    pub const LIME: Color = Color::new(0.00, 0.62, 0.18, 1.00);
    pub const DARKGREEN: Color = Color::new(0.00, 0.46, 0.17, 1.00);
    pub const SKYBLUE: Color = Color::new(0.40, 0.75, 1.00, 1.00);
    pub const BLUE: Color = Color::new(0.00, 0.47, 0.95, 1.00);
    pub const DARKBLUE: Color = Color::new(0.00, 0.32, 0.67, 1.00);
    pub const PURPLE: Color = Color::new(0.78, 0.48, 1.00, 1.00);
    pub const VIOLET: Color = Color::new(0.53, 0.24, 0.75, 1.00);
    pub const DARKPURPLE: Color = Color::new(0.44, 0.12, 0.49, 1.00);
    pub const BEIGE: Color = Color::new(0.83, 0.69, 0.51, 1.00);
    pub const BROWN: Color = Color::new(0.50, 0.42, 0.31, 1.00);
    pub const DARKBROWN: Color = Color::new(0.30, 0.25, 0.18, 1.00);
    pub const WHITE: Color = Color::new(1.00, 1.00, 1.00, 1.00);
    pub const BLACK: Color = Color::new(0.00, 0.00, 0.00, 1.00);
    pub const MAGENTA: Color = Color::new(1.00, 0.00, 1.00, 1.00);
    pub const TRANSPARENT: Color = Color::new(0.00, 0.00, 0.00, 0.00);
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self { r: arr[0], g: arr[1], b: arr[2], a: arr[3] }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

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

    /// Left (minimum x) edge.
    #[inline]
    pub const fn left(&self) -> f32 {
        self.x
    }

    /// Right (maximum x) edge.
    #[inline]
    pub const fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Top (minimum y) edge – handy for symmetry with `left/right`.
    #[inline]
    pub const fn top(&self) -> f32 {
        self.y
    }

    /// Bottom (maximum y) edge.
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

/// 2D camera for controlling the viewport.
#[derive(Clone, Debug)]
pub struct Camera2D {
    /// The point in world space the camera is looking at.
    pub target: Vec2,
    /// Zoom level (higher = more zoomed in).
    pub zoom: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
    /// Offset from the target in screen space.
    pub offset: Vec2,
    /// Optional render target for off-screen rendering.
    #[cfg(feature = "macroquad")]
    pub render_target: Option<crate::render_target::RenderTarget>,
    pub viewport: Option<(i32, i32, i32, i32)>,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            target: Vec2::ZERO,
            zoom: Vec2::ONE,
            rotation: 0.0,
            offset: Vec2::ZERO,
            #[cfg(feature = "macroquad")]
            render_target: None,
            viewport: None,
        }
    }
}

impl Camera2D {
    /// Creates a new camera with the given target and zoom.
    pub fn new(target: Vec2, zoom: Vec2) -> Self {
        Self {
            target,
            zoom,
            ..Default::default()
        }
    }

    /// Converts a world position to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let x = (world_pos.x - self.target.x) * self.zoom.x + self.offset.x;
        let y = (world_pos.y - self.target.y) * self.zoom.y + self.offset.y;
        Vec2::new(x, y)
    }

    /// Returns the world space position for a 2d camera screen space position.
    pub fn screen_to_world(&self, point: Vec2) -> Vec2 {
        let dims = self
            .viewport()
            .map(|(vx, vy, vw, vh)| Rect {
                x: vx as f32,
                y: crate::backend::screen_height() - (vy + vh) as f32,
                w: vw as f32,
                h: vh as f32,
            })
            .unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                w: crate::backend::screen_width(),
                h: crate::backend::screen_height(),
            });

        let point = vec2(
            (point.x - dims.x) / dims.w * 2. - 1.,
            1. - (point.y - dims.y) / dims.h * 2.,
        );
        let inv_mat = self.matrix().inverse();
        let transform = inv_mat.transform_point3(vec3(point.x, point.y, 0.));

        vec2(transform.x, transform.y)
    }

    fn matrix(&self) -> Mat4 {
        let mat_origin = Mat4::from_translation(vec3(-self.target.x, -self.target.y, 0.0));
        let mat_rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let invert_y = if self.render_target.is_some() {
            1.0
        } else {
            -1.0
        };
        let mat_scale = Mat4::from_scale(vec3(self.zoom.x, self.zoom.y * invert_y, 1.0));
        let mat_translation = Mat4::from_translation(vec3(self.offset.x, self.offset.y, 0.0));

        mat_translation * ((mat_scale * mat_rotation) * mat_origin)
    }

    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        self.viewport
    }
}

/// Re-export Texture2D from macroquad when using macroquad backend.
#[cfg(feature = "macroquad")]
pub use macroquad::prelude::Texture2D;
