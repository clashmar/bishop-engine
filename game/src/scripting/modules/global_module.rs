// game/src/scripting/modules/global_module.rs
use crate::scripting::lua_game_ctx::LuaGameCtx;
use engine_core::scripting::lua_constants::GLOBAL_FILE;
use engine_core::scripting::modules::lua_module::*;
use engine_core::scripting::script::Script;
use engine_core::ecs::component::*;
use mlua::prelude::LuaResult;
use mlua::MultiValue;
use engine_core::*;
use mlua::Function;
use mlua::Variadic;
use mlua::Value;
use mlua::Lua;

/// Lua module that exposes the engine's global modules to scripts.
#[derive(Default)]
pub struct GlobalModule;
register_lua_module!(GlobalModule);

impl LuaModule for GlobalModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        let global_table = lua.create_table()?;

        // global.call(name, method_name, ...)
        let call_fn = lua.create_function(|lua, args: Variadic<Value>| {
            // Extract global entity name and method name
            let mut iter = args.into_iter();
            
            let name = match iter.next() {
                Some(Value::String(s)) => s.to_str()?.to_owned(),
                _ => return Err(mlua::Error::RuntimeError(
                    "First argument must be the global entity name (string)".into()
                )),
            };

            let method = match iter.next() {
                Some(Value::String(s)) => s.to_str()?.to_owned(),
                _ => return Err(mlua::Error::RuntimeError(
                    "Second argument must be the method name (string)".into()
                )),
            };

            // Remaining args go to the method
            let method_args: Vec<Value> = iter.collect();

            // Execute synchronously since we need return values
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_state = ctx.game_state.borrow();
            let ecs = &game_state.game.ecs;
            let script_manager = &game_state.game.script_manager;

            // Find global entity by name
            let global_entity = {
                let global_store = ecs.get_store::<Global>();
                let name_store = ecs.get_store::<Name>();
                
                global_store.data.keys()
                    .find(|&&entity| {
                        name_store.get(entity)
                            .map(|n| n.0 == name)
                            .unwrap_or(false)
                    })
                    .copied()
            };

            let entity = global_entity.ok_or_else(|| {
                mlua::Error::RuntimeError(format!("Global entity '{}' not found", name))
            })?;

            // Get the script component
            let script_id = ecs.get_store::<Script>()
                .get(entity)
                .map(|s| s.script_id)
                .ok_or_else(|| {
                    mlua::Error::RuntimeError(
                        format!("Global entity '{}' has no script", name)
                    )
                })?;

            // Get the script instance
            let instance = script_manager.instances
                .get(&(entity, script_id))
                .ok_or_else(|| {
                    mlua::Error::RuntimeError(
                        format!("Script instance not found for global '{}'", name)
                    )
                })?;

            // Get the method function
            let func = instance.get::<Function>(method.clone())
                .map_err(|_| {
                    mlua::Error::RuntimeError(
                        format!("Method '{}' not found on global '{}'", method, name)
                    )
                })?;

            // Build call args with instance as first argument (self)
            let handle = Value::Table(instance.clone());
            let mut call_args = Vec::with_capacity(method_args.len() + 1);
            call_args.push(handle);
            call_args.extend(method_args);

            // Execute the call synchronously and return the result
            func.call::<MultiValue>(MultiValue::from_vec(call_args))
        })?;

        global_table.set("call", call_fn)?;

        // Set as global module
        lua.globals().set("global", global_table)?;

        Ok(())
    }
}

impl LuaApi for GlobalModule {
    fn emit_api(&self, out: &mut LuaApiWriter) {
        out.line("--- Global entity script API");
        out.line("global = {}");
        out.line("");
        out.line("--- Call a method on a global entity script");
        out.line("--- @param name string The name of the global entity");
        out.line("--- @param method string The method name to call");
        out.line("--- @param ... any Additional arguments to pass to the method");
        out.line("--- @return any Returns whatever the method returns");
        out.line("function global.call(name, method, ...) end");
        out.line("");
    }
}

register_lua_api!(GlobalModule, GLOBAL_FILE);