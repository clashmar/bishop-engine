// engine_core/src/script/script_system.rs
use crate::scripting::modules::entity_module::*;
use crate::game_global::drain_commands;
use crate::engine::Engine;
use engine_core::scripting::modules::lua_module::LuaModuleRegistry;
use engine_core::scripting::script_manager::ScriptManager;
use engine_core::scripting::lua_constants::*;
use engine_core::scripting::script::Script;
use engine_core::ecs::world_ecs::WorldEcs;
use mlua::prelude::LuaResult;
use engine_core::*;
use mlua::Function;
use mlua::Variadic;
use mlua::Value;
use mlua::Lua;

pub struct ScriptSystem;

impl ScriptSystem {
    /// Initialize the script system.
    pub fn init(lua: &Lua, script_manager: &mut ScriptManager) {
        // .engine module
        if let Err(e) = Self::register_engine_module(lua, script_manager) {
            onscreen_error!("Error registering engine module: {e}")
        };

        // Sub-modules
        for descriptor in inventory::iter::<LuaModuleRegistry> {
            // Build the concrete module and register it
            let module = (descriptor.ctor)();
            if let Err(e) = module.register(lua) {
                onscreen_error!("Lua module registration failed: {e}");
            }
        }
    }

    /// Call this once after the `Lua` instance has been created.
    fn register_engine_module(lua: &Lua, script_manager: &mut ScriptManager) -> LuaResult<()> {
        // Build the module
        let engine_mod = lua.create_table()?;

        // engine.call(name, ...)
        let engine_api = script_manager.engine_api.clone();
        let call_fn = lua.create_function(move |lua, args: Variadic<Value>| {
            engine_api.lua_call(lua, args)
        })?;
        engine_mod.set(ENGINE_CALL, call_fn)?;

        // Convenience wrappers (engine.log, engine.wait, …)
        let engine_api = script_manager.engine_api.clone();
        for name in engine_api.callbacks.lock().unwrap().keys() {
            let fn_name = name.clone();
            let api = engine_api.clone();
            let wrapper = lua.create_function(move |lua, args: Variadic<Value>| {
                let mut full = vec![Value::String(lua.create_string(&fn_name)?)];
                full.extend_from_slice(&args);
                api.lua_call(lua, Variadic::from(full))
            })?;
            engine_mod.set(name.clone(), wrapper)?;
        }

        // engine.on(event, handler)
        let engine_api = script_manager.engine_api.clone();
        let on_fn = lua.create_function(move |_, (event, handler): (String, Function)| {
            engine_api.listeners
                .lock()
                .unwrap()
                .entry(event)
                .or_default()
                .push(handler);
            Ok(())
        })?;
        engine_mod.set(ENGINE_ON, on_fn)?;

        // engine.emit(event, …)
        let engine_api = script_manager.engine_api.clone();
        let emit_fn = lua.create_function(move |_lua, (event, args): (String, Variadic<Value>)| {
            let map = engine_api.listeners.lock().unwrap();
            if let Some(callbacks) = map.get(&event) {
                for cb in callbacks {
                    if let Err(e) = cb.call::<()>(args.clone()) {
                        onscreen_error!("Lua listener error for event '{}': {}", event, e);
                    }
                }
            }
            Ok(())
        })?;
        engine_mod.set(ENGINE_EMIT, emit_fn)?;

        lua.globals().set(ENGINE, engine_mod.clone())?;
        lua.register_module(ENGINE, &engine_mod)?;
        Ok(())
    }

    /// Process all Lua commands to the `GameState`.
    /// Called once per frame, before any Lua script runs.
    pub fn process_commands(engine: &mut Engine) {
        // Drain the command queue and apply each command
        for mut cmd in drain_commands() {
            cmd.execute(engine);
        }
    }
}

// Load all scripts for the given ecs.
pub fn load_scripts(
    lua: &Lua,
    world_ecs: &mut WorldEcs, 
    script_manager: &mut ScriptManager
) -> LuaResult<()> {
    let script_store = world_ecs.get_store_mut::<Script>();

    for (_entity, script) in script_store.data.iter_mut() {
        if !script_manager.tables.contains_key(&script.script_id) {
            script.load(lua, script_manager)?
        }
    }

    Ok(())
}

// Run all scripts for the given ecs.
pub fn run_scripts(
    dt: f32,
    world_ecs: &WorldEcs, 
    script_manager: &ScriptManager,
    lua: &Lua,
) -> LuaResult<()> {
    let script_store = world_ecs.get_store::<Script>();
    for (entity, script) in script_store.data.iter() {
        if let Some(update) = script_manager.update_fns.get(&script.script_id) {
            let table = script_manager.tables.get(&script.script_id).unwrap();
            let handle = lua_entity_handle(lua, *entity)?;
            table.set(ENTITY, handle)?;
            update.call::<()>((table, dt))?
        }
    }
    Ok(())
}