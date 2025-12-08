// engine_core/src/script/runtime_resources.rs
use crate::input::input_snapshot::InputSnapshot;
use crate::ecs::world_ecs::WorldEcs;
use std::sync::Mutex;
use std::sync::Arc;

/// All data that must be reachable from Lua.
#[derive(Clone)]
pub struct RuntimeResources {
    pub world: Arc<Mutex<WorldEcs>>,
    pub input: Arc<Mutex<InputSnapshot>>,
}