// engine_core/src/script/script_system.rs
use crate::scripting::modules::entity_module::*;
use crate::game_global::drain_commands;
use crate::engine::Engine;
use engine_core::scripting::modules::lua_module::LuaModuleRegistry;
use engine_core::scripting::script_manager::ScriptManager;
use engine_core::scripting::lua_constants::*;
use engine_core::storage::path_utils::*;
use engine_core::scripting::script::*;
use engine_core::ecs::ecs::Ecs;
use mlua::prelude::LuaResult;
use engine_core::*;
use mlua::Function;
use std::sync::Arc;
use mlua::Lua;
use std::fs;

pub struct ScriptSystem;

impl ScriptSystem {
    /// Initialize the script system.
    pub fn init(lua: &Lua) {
        // Registers the `engine` module that some other modules extend
        if let Err(e) = Self::register_engine_module(lua) {
            onscreen_error!("Error registering engine module: {e}")
        };
        
        // Run main.lua after registering `engine``
        if let Err(e) = Self::load_main(lua) {
            onscreen_error!("Main failed: {e}");
        }

        // Sub-modules
        for descriptor in inventory::iter::<LuaModuleRegistry> {
            let module = (descriptor.ctor)();
            if let Err(e) = module.register(lua) {
                onscreen_error!("Lua module registration failed: {e}");
            }
        }
    }

    /// Loads and executes main.lua if present.
    fn load_main(lua: &Lua) -> LuaResult<()> {
        // TODO: get folder to point to correct game automatically
        let main_path = scripts_folder("Demo").join("main.lua");
        let src = fs::read_to_string(main_path)
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
        lua.load(&src).exec()
    }

    /// Called during init.
    fn register_engine_module(lua: &Lua) -> LuaResult<()> {
        let engine_mod = lua.create_table()?;
        lua.globals().set(ENGINE, engine_mod.clone())?;
        lua.register_module(ENGINE, &engine_mod)?;
        Ok(())
    }

    /// Runs all lua scripts in the game.
    pub fn run_scripts(
        dt: f32,
        engine: &mut Engine,
    ) -> LuaResult<()> {
        // Run all pending init functionsm queued in load phase
        let pending_init: Vec<_> = {
            let game_state = engine.game_state.borrow();
            game_state.game.script_manager.pending_inits.clone()
        };

        {
            let mut game_state = engine.game_state.borrow_mut();
            game_state.game.script_manager.pending_inits.clear();
        }

        for (entity, script_id) in pending_init {
            let (instance, init_fn) = {
                let game_state = engine.game_state.borrow();
                let sm = &game_state.game.script_manager;

                let instance = sm.get_instance(entity, script_id)?.clone();
                let init = instance.get::<Function>(INIT).ok();

                (instance, init)
            };

            if let Some(init_fn) = init_fn {
                init_fn.call::<()>(&instance)?;
                // Process commands immediately after init completes
                Self::process_commands(engine);
            }
        }

        // Run all update functions on scripts
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
                let script_manager = &game_state.game.script_manager;
                
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

    /// Initializes all needed scripts in the game.
    pub fn load_scripts(
        lua: &Lua,
        ecs: &mut Ecs,
        script_manager: &mut ScriptManager,
    ) -> LuaResult<()> {
        let script_store = ecs.get_store_mut::<Script>();

        for (entity, script) in script_store.data.iter_mut() {
            let created;

            {
                let (instance, was_created) =
                    script_manager.get_or_create_instance(lua, *entity, script.script_id)?;

                created = was_created;

                // Always expose the entity handle
                let handle = lua_entity_handle(lua, *entity)?;
                instance.set(ENTITY, handle)?;
            }

            // Script manager is free again here
            if created {
                // Sync first
                script.sync_to_lua(lua, script_manager, *entity)?;

                let instance = script_manager.get_instance(*entity, script.script_id)?;

                // Queue the init function for a script if present
                if let Ok(_init_fn) = instance.get::<Function>(INIT) {
                    script_manager
                        .pending_inits
                        .push((*entity, script.script_id));
                }
            }
        }

        Ok(())
    }
}



