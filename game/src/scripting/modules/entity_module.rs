// game/src/scripting/modules/entity_module.rs
use crate::scripting::commands::lua_command::*;
use crate::game_global::push_command;
use engine_core::scripting::modules::lua_module::LuaModule;
use engine_core::ecs::entity::Entity;
use engine_core::register_lua_module;
use mlua::UserDataMethods;
use mlua::UserData;
use std::sync::mpsc;
use mlua::Value;
use mlua::Lua;

/// Lua module that exposes a constructor for `EntityHandle`.
#[derive(Default)]
pub struct EntityModule;
register_lua_module!(EntityModule);

impl LuaModule for EntityModule {
    fn register(&self, lua: &Lua) -> mlua::Result<()> {
        let factory = lua.create_function(|_, id: usize| {
            Ok(EntityHandle {
                entity: Entity(id),
            })
        })?;
        lua.globals().set("entity", factory)?;
        Ok(())
    }
}

/// A thin wrapper that carries an Entity id.
#[derive(Clone)]
pub struct EntityHandle {
    pub entity: Entity,
}

/// Returns `Ok(Value)` if the component exists, otherwise a Lua error.
pub fn read_component<'lua>(
    lua: &'lua mlua::Lua,
    state: &crate::game_state::GameState,
    entity_id: usize,
    comp_name: &str,
) -> mlua::Result<Value> {
    // Find the registry entry for the component name
    let reg = engine_core::ecs::component_registry::COMPONENTS
        .iter()
        .find(|r| r.type_name == comp_name)
        .ok_or_else(|| mlua::Error::RuntimeError(format!("Component '{}' not known", comp_name)))?;

    // Get a mutable reference to the current world (the same as in GetComponentCmd)
    let world = &state.game.current_world().world_ecs;

    // Check that the entity actually has the component
    if !(reg.has)(world, Entity(entity_id)) {
        return Err(mlua::Error::RuntimeError(format!(
            "Entity {} has no {} component",
            entity_id, comp_name
        )));
    }

    // Clone the boxed component and convert it to a Lua value.
    // `reg.clone` returns `Box<dyn Any>`; we then hand it to `to_lua`.
    let boxed = (reg.clone)(world, Entity(entity_id));
    (reg.to_lua)(lua, &*boxed)
}

impl UserData for EntityHandle {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        // e:has("Component")
        methods.add_method("has", |_, this, comp_name: String| {
            let (tx, rx) = mpsc::channel();
            push_command(Box::new(GetComponentCmd{ 
                entity: *this.entity, 
                comp_name, 
                responder: tx,
            }));
            match rx.recv() {
                Ok(Ok(_)) => Ok(true),
                _ => Ok(false),
            }
        });

        // e:get("Component")
        methods.add_method("get", |_lua, this, comp_name: String| {
            crate::game_global::with_game_state(|state| {
                read_component(
                    &state.game.script_manager.lua,
                    state,
                    *this.entity,
                    &comp_name,
                )
            })
        });

        // e:set("Component", Value)
        methods.add_method("set", |_lua, this, (comp_name, value): (String, Value)| {
            push_command(Box::new(SetComponentCmd { 
                entity: *this.entity, 
                comp_name, 
                value,
            }));
            Ok(())
        });

        // convenience: `entity.id` (read‑only)
        methods.add_method("id", |_, this, ()| Ok(*this.entity));
    }
    
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(*this.entity));
    }
    
    fn register(registry: &mut mlua::UserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}