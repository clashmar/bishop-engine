//! State management modules for wgpu backend.

mod graphics;
mod input;
mod time;

pub use graphics::{GraphicsState, GraphicsStateError};
pub use input::InputState;
pub use time::TimeState;
