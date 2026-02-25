//! Macroquad backend legacy functions.
//!
//! This module provides free functions that wrap macroquad's global state API until they can be deleted.
//! These are only available when the `macroquad` feature is enabled.

mod camera;
mod draw;
mod input;
mod text;
mod texture;
mod time;
mod window;

pub use camera::*;
pub use draw::*;
pub use input::*;
pub use text::*;
pub use texture::*;
pub use time::*;
pub use window::*;
