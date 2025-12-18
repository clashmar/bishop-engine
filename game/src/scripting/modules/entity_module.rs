// game/src/scripting/modules/entity_module.rs
use crate::scripting::commands::lua_command::*;
use crate::scripting::lua_game_ctx::LuaGameCtx;
use crate::game_global::push_command;
use crate::scripting::lua_helpers::*;
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::scripting::modules::lua_module::*;
use engine_core::scripting::lua_constants::*;
use engine_core::ecs::entity::Entity;
use mlua::prelude::LuaResult;
use mlua::UserDataRegistry;
use mlua::UserDataMethods;
use mlua::Variadic;
use mlua::UserData;
use engine_core::*;
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

/// Build a Lua userdata object that wraps `Entity`.
pub fn lua_entity_handle<'lua>(lua: &'lua Lua, entity: Entity) -> LuaResult<Value> {
    // `EntityHandle` is `Clone`, so we can move it into the userdata.
    let handle = EntityHandle { entity };
    lua.create_userdata(handle).map(Value::UserData)
}

impl UserData for EntityHandle {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        let methods_vec = [
            EntityHandleMethod::Get(GetMethod),
            EntityHandleMethod::Set(SetMethod),
            EntityHandleMethod::Has(HasMethod),
        ];

        for m in &methods_vec {
            m.register(methods);
        }
    }
    
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get(ID, |_, this| Ok(*this.entity));
    }
    
    fn register(registry: &mut UserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}

register_lua_api!(EntityModule, ENTITY_FILE);

impl LuaApi for EntityModule {
    fn emit_api(&self, out: &mut LuaApiWriter) {
        // Define entity class
        out.line("---@class Entity");
        out.line("---@field id integer");
        out.line("local Entity = {}");
        out.line("");

        // Emit each registered method
        let methods_vec = [
            EntityHandleMethod::Get(GetMethod),
            EntityHandleMethod::Set(SetMethod),
            EntityHandleMethod::Has(HasMethod),
        ];
        for m in methods_vec.iter() {
            m.emit_api(out);
        }

        out.line("return Entity");
    }
}

pub enum EntityHandleMethod {
    Get(GetMethod),
    Set(SetMethod),
    Has(HasMethod),
}

impl LuaMethod<EntityHandle> for EntityHandleMethod {
    fn register<M: UserDataMethods<EntityHandle>>(&self, methods: &mut M) {
        match self {
            EntityHandleMethod::Get(m) => m.register(methods),
            EntityHandleMethod::Set(m) => m.register(methods),
            EntityHandleMethod::Has(m) => m.register(methods),
        }
    }

    fn emit_api(&self, out: &mut LuaApiWriter) {
        match self {
            EntityHandleMethod::Get(m) => m.emit_api(out),
            EntityHandleMethod::Set(m) => m.emit_api(out),
            EntityHandleMethod::Has(m) => m.emit_api(out),
        }
    }
}

/// Method: `entity:get("Component")`
pub struct GetMethod;
impl LuaMethod<EntityHandle> for GetMethod {
    fn register<M: UserDataMethods<EntityHandle>>(&self, methods: &mut M) {
        methods.add_method(GET, |lua, this, comp_name: String| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_state = ctx.game_state.borrow();
            let world_ecs = &game_state.game.current_world().world_ecs;
            let entity = this.entity;

            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
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
                    comp_name
                )))
            }
        });
    }

    fn emit_api(&self, out: &mut LuaApiWriter) {
        out.line("-- Component getters");
        for reg in COMPONENTS.iter() {
            out.line(&format!(
                "---@overload fun(self: Entity, component: \"{}\"): {}",
                reg.type_name, reg.type_name
            ));
        }
        out.line("---@param component string");
        out.line("---@return table|nil");
        out.line(&format!("function Entity:{}(component) end", GET));
        out.line("");
    }
}

/// Method: `entity:set("Component", value)`
pub struct SetMethod;
impl LuaMethod<EntityHandle> for SetMethod {
    fn register<M: UserDataMethods<EntityHandle>>(&self, methods: &mut M) {
        methods.add_method(SET, |_lua, this, (comp_name, value): (String, Value)| {
            push_command(Box::new(SetComponentCmd {
                entity: *this.entity,
                comp_name,
                value,
            }));
            Ok(())
        });

        // Typed setters
        for reg in COMPONENTS.iter() {
            let comp_name = reg.type_name.to_string();
            let fn_name = format!("{}_{}", SET, to_snake_case(reg.type_name));
            methods.add_method(fn_name.as_str(), move |_lua, this, value: Value| {
                push_command(Box::new(SetComponentCmd {
                    entity: *this.entity,
                    comp_name: comp_name.clone(),
                    value,
                }));
                Ok(())
            });
        }
    }

    fn emit_api(&self, out: &mut LuaApiWriter) {
        out.line("-- Generic set method");
        out.line("---@param component string");
        out.line("---@see ComponentId");
        out.line("---@param value table");
        out.line(&format!("function Entity:{}(component, value) end", SET));
        out.line("");

        out.line("-- Typed component setters");
        for reg in COMPONENTS.iter() {
            let type_name = reg.type_name;
            let fn_name = to_snake_case(type_name);
            out.line(&format!("---@param self Entity"));
            out.line(&format!("---@param v {}", type_name));
            out.line(&format!("function Entity:{}_{}(v) end", SET, fn_name));
            out.line("");
        }
    }
}

/// Method: `entity:has(...)`, `has_any`, `has_all`
pub struct HasMethod;
impl LuaMethod<EntityHandle> for HasMethod {
    fn register<M: UserDataMethods<EntityHandle>>(&self, methods: &mut M) {
        // entity:has
        methods.add_method(HAS, |lua, this, comp_name: String| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let binding = ctx.game_state.borrow();
            let world_ecs = &binding.game.current_world().world_ecs;
            Ok(COMPONENTS.iter().find(|r| r.type_name == comp_name).map_or(false, |r| (r.has)(world_ecs, this.entity)))
        });

        // entity:has_any
        methods.add_method(HAS_ANY, |lua, this, comps: Variadic<String>| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let binding = ctx.game_state.borrow();
            let world_ecs = &binding.game.current_world().world_ecs;
            for comp_name in comps.iter() {
                if let Some(r) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
                    if (r.has)(world_ecs, this.entity) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        });

        // entity:has_all
        methods.add_method(HAS_ALL, |lua, this, comps: Variadic<String>| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let binding = ctx.game_state.borrow();
            let world_ecs = &binding.game.current_world().world_ecs;
            for comp_name in comps.iter() {
                if let Some(r) = COMPONENTS.iter().find(|r| r.type_name == comp_name) {
                    if !(r.has)(world_ecs, this.entity) { return Ok(false); }
                } else { return Ok(false); }
            }
            Ok(true)
        });
    }

    fn emit_api(&self, out: &mut LuaApiWriter) {
        // has
        out.line("---@param component string");
        out.line("---@see ComponentId");
        out.line("---@return boolean");
        out.line(&format!("function Entity:{}(component) end", HAS));
        out.line("");

        // has_any
        out.line("---@param ... string");
        out.line("---@see ComponentId");
        out.line("---@return boolean");
        out.line(&format!("function Entity:{}(...) end", HAS_ANY));
        out.line("");

        // has_all
        out.line("---@param ... string");
        out.line("---@see ComponentId");
        out.line("---@return boolean");
        out.line(&format!("function Entity:{}(...) end", HAS_ALL));
        out.line("");
    }
}