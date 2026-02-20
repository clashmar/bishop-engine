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
//! use bishop::*;
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

pub mod draw;
pub mod input;
pub mod text;
pub mod types;

#[cfg(feature = "macroquad")]
pub mod macroquad_impl;

pub use draw::*;
pub use input::*;
pub use text::*;
pub use types::*;

#[cfg(feature = "macroquad")]
pub use macroquad_impl::MacroquadContext;

/// Combined context trait for widgets that need input, drawing, and text.
pub trait BishopContext: Input + Draw + Text {}

impl<T: Input + Draw + Text> BishopContext for T {}
