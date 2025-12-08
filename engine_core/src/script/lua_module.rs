// engine_core/src/script/lua_module.rs
use mlua::Lua;

/// Every system that wants to expose Lua functions implements this.
pub trait LuaModule {
    fn register(&self, lua: &Lua) -> mlua::Result<()>;
}