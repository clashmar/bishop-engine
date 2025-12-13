// game/src/scripting/modules/entity_module.rs
use crate::scripting::commands::lua_command::*;
use crate::scripting::lua_game_ctx::LuaGameCtx;
use crate::game_global::push_command;
use engine_core::scripting::lua_constants::ENTITY;
use engine_core::scripting::modules::lua_module::LuaModule;
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::scripting::lua_constants::*;
use engine_core::ecs::entity::Entity;
use engine_core::register_lua_module;
use mlua::prelude::LuaResult;
use mlua::UserDataRegistry;
use mlua::UserDataMethods;
use mlua::Variadic;
use mlua::UserData;
use mlua::Value;
use mlua::Lua;

/// Lua module that exposes a constructor for `EntityHandle`.
#[derive(Default)]
pub struct EntityModule;
register_lua_module!(EntityModule);

impl LuaModule for EntityModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        // Wraps an entity(id) in a lua EntityHandle
        let factory = lua.create_function(|_, id: usize| {
            Ok(EntityHandle {
                entity: Entity(id),
            })
        })?;
        lua.globals().set(ENTITY, factory)?;
        Ok(())
    }
}

/// A lua wrapper that carries an Entity id.
#[derive(Clone)]
pub struct EntityHandle {
    pub entity: Entity,
}

/// Build a Lua userdata that wraps `entity`.
pub fn lua_entity_handle<'lua>(lua: &'lua Lua, entity: Entity) -> LuaResult<Value> {
    // `EntityHandle` is `Clone`, so we can move it into the userdata.
    let handle = EntityHandle { entity };
    lua.create_userdata(handle).map(Value::UserData)
}

impl UserData for EntityHandle {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        // entity:get("Component")
        methods.add_method(GET, |lua, this, comp_name: String| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_state = ctx.game_state.borrow();
            let world_ecs = &game_state.game.current_world().world_ecs;
            let entity = this.entity;
            let result = if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
                if (reg.has)(world_ecs, entity) {
                    let boxed = (reg.clone)(world_ecs, entity);
                    (reg.to_lua)(lua, &*boxed)
                } else {
                    Err(mlua::Error::RuntimeError(format!(
                        "Entity {:?} has no {} component",
                        entity, comp_name
                    )))
                }
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Component '{}' not known", 
                    comp_name,
                )))
            };
            result
        });

        // entity:set("Component", Value)
        methods.add_method(SET, |_lua, this, (comp_name, value): (String, Value)| {
            push_command(Box::new(SetComponentCmd { 
                entity: *this.entity, 
                comp_name, 
                value,
            }));
            Ok(())
        });

        // entity:has("Component")
        methods.add_method(HAS, |lua, this, comp_name: String| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_state = ctx.game_state.borrow();
            let world_ecs = &game_state.game.current_world().world_ecs;
            let entity = this.entity;

            let has = if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
                (reg.has)(world_ecs, entity)
            } else {
                // Unknown component
                false
            };
            Ok(has)
        });

        // entity:has_any("ComponentA", "ComponentB")
        methods.add_method(HAS_ANY, |lua, this, comps: Variadic<String>| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_state = ctx.game_state.borrow();
            let world_ecs = &game_state.game.current_world().world_ecs;
            let entity = this.entity;

            // Return true on first match
            for comp_name in comps.iter() {
                if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
                    if (reg.has)(world_ecs, entity) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        });

        // entity:has_all("ComponentA", "ComponentB")
        methods.add_method(HAS_ALL, |lua, this, comps: Variadic<String>| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_state = ctx.game_state.borrow();
            let world_ecs = &game_state.game.current_world().world_ecs;
            let entity = this.entity;

            // Return false as soon as a component is missing or unknown
            for comp_name in comps.iter() {
                if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
                    if !(reg.has)(world_ecs, entity) {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Ok(true)
        });
    }

    
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get(ID, |_, this| Ok(*this.entity));
    }
    
    fn register(registry: &mut UserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}

