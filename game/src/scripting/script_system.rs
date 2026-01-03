// engine_core/src/script/script_system.rs
use crate::scripting::modules::entity_module::*;
use crate::game_global::drain_commands;
use crate::engine::Engine;
use engine_core::scripting::modules::lua_module::LuaModuleRegistry;
use engine_core::scripting::script_manager::ScriptManager;
use engine_core::scripting::lua_constants::*;
use engine_core::scripting::script::*;
use engine_core::ecs::ecs::Ecs;
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

    /// Runs all lua scripts in the game.
    pub fn run_scripts(
        dt: f32,
        engine: &mut Engine,
    ) -> LuaResult<()> {
        let entities_and_scripts: Vec<_> = {
            let game_state = engine.game_state.borrow();
            let ctx = game_state.game.ctx();
            let ecs = ctx.ecs;
            let script_store = ecs.get_store::<Script>();
            
            // Collect all entities that have scripts
            script_store.data.iter()
                .map(|(entity, script)| (*entity, script.script_id.clone()))
                .collect()
        };

        for (entity, script_id) in entities_and_scripts {
            let (update, instance) = {
                let game_state = engine.game_state.borrow();
                let ctx = game_state.game.ctx();
                let script_manager = ctx.script_manager;
                
                if let Some(update) = script_manager.update_fns.get(&script_id) {
                    // Instance should exist and be setup already
                    if let Some(instance) = script_manager.instances.get(&(entity, script_id)) {
                        // Clone before dropping the borrow
                        (Some(update.clone()), Some(instance.clone()))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            };
                
            // Make sure game_state borrow is dropped
            if let (Some(update), Some(instance)) = (update, instance) {
                // Execute the script's update function
                update.call::<()>((instance, dt))?;
                
                // Process commands immediately after this script completes
                Self::process_commands(engine);
            }
        }

        Ok(())
    }

    /// Process all Lua commands to the `Engine`.
    pub fn process_commands(engine: &mut Engine) {
        // Drain the command queue and apply each command
        for mut cmd in drain_commands() {
            cmd.execute(engine);
        }
    }

    // Load all scripts for the given ecs.
    pub fn load_scripts(
        lua: &Lua,
        ecs: &mut Ecs, 
        script_manager: &mut ScriptManager
    ) -> LuaResult<()> {
        let script_store = ecs.get_store_mut::<Script>();

        for (entity, script) in script_store.data.iter_mut() {
            script.load(lua, script_manager, *entity)?; // TODO: load every frame?
            if let Some(instance) = script_manager.instances.get(&(*entity, script.script_id)) {
                let handle = lua_entity_handle(&lua, *entity)?;
                instance.set(ENTITY, handle)?; // TODO: Can this be done automatically?
            } 
        }

        Ok(())
    }
}



