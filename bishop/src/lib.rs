//! Bishop - Backend abstraction traits for the bishop-engine.
//!
//! This crate provides trait abstractions for input, drawing, and text rendering
//! that can be implemented by different backends (macroquad, winit+wgpu, etc.).
//!
//! # Features
//!
//! - `macroquad` (default): Enables the macroquad backend implementation.
//!
//! # Example
//!
//! ```ignore
//! use bishop::prelude::*;
//!
//! fn draw_button<C: BishopContext>(ctx: &mut C, rect: Rect, label: &str) -> bool {
//!     let mouse = ctx.mouse_position();
//!     let hovered = rect.contains(Vec2::new(mouse.0, mouse.1));
//!
//!     let bg_color = if hovered { Color::GRAY } else { Color::BLACK };
//!     ctx.draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
//!     ctx.draw_text(label, rect.x + 5.0, rect.y + 20.0, 16.0, Color::WHITE);
//!
//!     ctx.is_mouse_button_pressed(MouseButton::Left) && hovered
//! }
//! ```

pub mod backend;
pub mod camera;
pub mod draw;
pub mod input;
pub mod material;
pub mod text;
pub mod time;
pub mod types;
pub mod window;

#[cfg(feature = "macroquad")]
pub mod macroquad;

pub use camera::*;
pub use draw::*;
pub use input::*;
pub use text::*;
pub use time::*;
pub use types::*;
pub use window::*;

#[cfg(feature = "macroquad")]
pub use macroquad::MacroquadContext;

/// Combined context trait for widgets that need input, drawing, text, camera, window, and time.
pub trait BishopContext: Input + Draw + Text + Camera + Window + Time {}

impl<T: Input + Draw + Text + Camera + Window + Time> BishopContext for T {}

/// Prelude module for convenient glob imports.
///
/// # Example
///
/// ```ignore
/// use bishop::prelude::*;
/// ```
pub mod prelude {
    pub use crate::backend::*;
    pub use crate::camera::*;
    pub use crate::draw::*;
    pub use crate::input::*;
    pub use crate::material::*;
    pub use crate::text::*;
    pub use crate::time::*;
    pub use crate::types::*;
    pub use crate::window::*;
    pub use crate::BishopContext;
    pub use glam::{Vec2, vec4};

    #[cfg(feature = "macroquad")]
    pub use crate::macroquad::MacroquadContext;
}
