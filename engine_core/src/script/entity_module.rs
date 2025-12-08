// engine_core/src/script/entity_module.rs
use crate::ecs::entity::Entity;
use crate::script::lua_module::LuaModule;
use crate::script::entity_handle::EntityHandle;
use mlua::Lua;
use crate::ecs::world_ecs::WorldEcs;
use std::sync::Mutex;
use std::sync::Arc;

/// Small wrapper that implements the `LuaModule` trait (see §3).
pub struct EntityModule {
    pub world: Arc<Mutex<WorldEcs>>,
}

impl LuaModule for EntityModule {
    fn register(&self, lua: &Lua) -> mlua::Result<()> {
        let world = self.world.clone();
        let factory = lua.create_function(move |_, id: usize| {
            Ok(EntityHandle {
                entity: Entity(id),
                world: world.clone(),
            })
        })?;
        lua.globals().set("entity", factory)?;
        Ok(())
    }
}