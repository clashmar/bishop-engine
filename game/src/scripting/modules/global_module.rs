// game/src/scripting/modules/global_module.rs
use engine_core::scripting::modules::lua_module::*;
use mlua::prelude::LuaResult;
use engine_core::*;
use mlua::Lua;

/// Lua module that exposes the engine’s global modules to scripts.
#[derive(Default)]
pub struct GlobalModule;
register_lua_module!(GlobalModule);

impl LuaModule for GlobalModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        
        Ok(())
    }
}

impl LuaApi for GlobalModule {
    fn emit_api(&self, out: &mut LuaApiWriter) {
        // TODO: impl
    }
}

register_lua_api!(GlobalModule);