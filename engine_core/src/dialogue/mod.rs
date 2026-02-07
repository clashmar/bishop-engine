// engine_core/src/dialogue/mod.rs

pub mod dialogue_data;
pub mod dialogue_config;
pub mod dialogue_manager;
pub mod speech_bubble;
pub mod speech_system;
pub mod speech_renderer;

pub use dialogue_data::*;
pub use dialogue_config::*;
pub use dialogue_manager::*;
pub use speech_bubble::*;
pub use speech_system::*;
pub use speech_renderer::*;
