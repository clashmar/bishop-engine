// game/src/scripting/modules/global_module.rs
use crate::scripting::commands::lua_command::CallGlobalCmd;
use crate::game_global::push_command;
use engine_core::scripting::modules::lua_module::*;
use engine_core::scripting::lua_constants::*;
use mlua::prelude::LuaResult;
use std::sync::mpsc;
use engine_core::*;
use mlua::Variadic;
use mlua::Value;
use mlua::Lua;
use mlua::Table;

/// Lua module that exposes the engine’s global modules to scripts.
#[derive(Default)]
pub struct GlobalModule;
register_lua_module!(GlobalModule);

impl LuaModule for GlobalModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        // Get the existing engine table
        let engine_tbl = lua.globals().get::<Table>(ENGINE)?;

        // Global submodule
        let global_tbl = lua.create_table()?;

        // engine.global.get(name)
        // let get = {
        //     let lua = lua.clone();
        //     lua.create_function(move |_lua, name: String| {
        //         // Look up the value in the global map
        //         with_game_state_mut(|game_state| {
        //             let map = game_state.global_modules.borrow();
        //             if let Some(val) = map.get(&name) {
        //                 // Return a clone of the value
        //                 Ok(val.clone())
        //             } else {
        //                 Err(mlua::Error::RuntimeError(format!(
        //                     "Global '{}' not found.",
        //                     name
        //                 )))
        //             }
        //         })
        //     })?
        // };
        // global_tbl.set("get", get)?;

        // engine.global.call(name, method, …)
        let call = {
            let lua = lua.clone();
            lua.create_function(move |_lua, (name, method, args): (String, String, Variadic<Value>)| {
                let (tx, rx) = mpsc::channel();
                push_command(Box::new(CallGlobalCmd {
                        name,
                        method,
                        args: args.into_iter().collect(),
                        responder: tx,
                    }));
                rx.recv().unwrap_or_else(|_| Err(mlua::Error::RuntimeError(
                    "Command queue closed.".into()
                )))
            })?
        };

        global_tbl.set(ENGINE_CALL, call)?;
        engine_tbl.set(GLOBAL, global_tbl)?;
        Ok(())
    }
}

impl LuaApiModule for GlobalModule {
    fn emit_api(&self, out: &mut LuaApiWriter) {
        // TODO: impl
    }
}

register_lua_api!(GlobalModule);