// engine_core/src/scripting/modules/lua_module.rs
use mlua::prelude::LuaResult;
use mlua::Lua;

/// Every system that wants to expose Lua functions implements this.
pub trait LuaModule {
    /// Registers the module’s functions, types and globals with the given Lua state.
    fn register(&self, lua: &Lua) -> LuaResult<()>;
}

/// Registry that the inventory crate will collect.                  
pub struct LuaModuleRegistry {
    /// Called once for every module during start‑up.
    pub ctor: fn() -> Box<dyn LuaModule>,
}

// Collect all descriptors into a slice that lives for the whole program.
inventory::collect!(LuaModuleRegistry);

/// Registers 
#[macro_export]
macro_rules! register_lua_module {
    ($ty:ty) => {
        inventory::submit! {
            $crate::scripting::modules::lua_module::LuaModuleRegistry {
                ctor: || Box::new(<$ty>::default()),
            }
        }
    };
}