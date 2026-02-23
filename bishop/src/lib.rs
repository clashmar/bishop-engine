//! Bishop - Backend abstraction traits for the bishop-engine.
//!
//! This crate provides trait abstractions for input, drawing, and text rendering
//! that can be implemented by different backends (macroquad, winit+wgpu, etc.).
//!
//! # Backend Support
//!
//! The `BishopContext` trait can be implemented for any backend:
//! - Graphics backends (macroquad, wgpu) implement full rendering
//! - Console backends can implement with text-based or stub graphics
//! - Headless backends can implement with no-op rendering for testing
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

#[cfg(feature = "wgpu")]
pub mod wgpu;

pub use camera::*;
pub use draw::*;
pub use input::*;
pub use text::*;
pub use time::*;
pub use types::*;
pub use window::*;

#[cfg(feature = "macroquad")]
pub use macroquad::MacroquadContext;

#[cfg(feature = "wgpu")]
pub use wgpu::WgpuContext;

/// Combined context trait for widgets that need input, drawing, text, camera, window, and time.
pub trait BishopContext: Input + Draw + Text + Camera + Window + Time {}

impl<T: Input + Draw + Text + Camera + Window + Time> BishopContext for T {}

/// Trait for applications that can be run by bishop.
pub trait BishopApp {
    /// Called once per frame. The app handles its own update/render logic.
    fn frame(&mut self, ctx: &mut impl BishopContext) -> impl std::future::Future<Output = ()>;
}

/// Runs the main loop for a BishopApp.
#[cfg(feature = "macroquad")]
pub async fn run<A, C>(app: &mut A, ctx: &mut C)
where
    A: BishopApp,
    C: BishopContext,
{
    loop {
        ctx.update();
        app.frame(ctx).await;
        ::macroquad::prelude::next_frame().await;
    }
}

/// The context type for the active graphics backend.
///
/// This is a type alias that resolves to:
/// - `MacroquadContext` when the `macroquad` feature is enabled (default)
/// - `WgpuContext` when the `wgpu` feature is enabled
///
/// Use this at application entry points (main.rs) to create the context.
/// For function parameters, prefer `impl BishopContext` for flexibility.
#[cfg(feature = "macroquad")]
pub type PlatformContext = macroquad::MacroquadContext;

#[cfg(feature = "wgpu")]
pub type PlatformContext = wgpu::WgpuContext;

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
    pub use crate::BishopApp;
    pub use crate::BishopContext;
    pub use glam::{Vec2, vec4};

    #[cfg(feature = "macroquad")]
    pub use crate::run;

    #[cfg(feature = "macroquad")]
    pub use crate::macroquad::MacroquadContext;

    #[cfg(feature = "wgpu")]
    pub use crate::wgpu::WgpuContext;

    #[cfg(any(feature = "macroquad", feature = "wgpu"))]
    pub use crate::PlatformContext;
}
