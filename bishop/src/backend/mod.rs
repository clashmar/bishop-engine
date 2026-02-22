//! Global backend functions using thread-local state.
//!
//! This module provides free functions that use thread-local state, similar to macroquad's API.
//! Call `update()` at the start of each frame to capture input state.

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
