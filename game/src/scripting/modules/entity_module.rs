// game/src/scripting/modules/entity_module.rs
use crate::scripting::commands::lua_command::*;
use crate::game_global::push_command;
use crate::engine::LuaGameCtx;
use engine_core::scripting::modules::lua_module::LuaModule;
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::ecs::entity::Entity;
use engine_core::register_lua_module;
use mlua::prelude::LuaResult;
use mlua::UserDataMethods;
use mlua::AnyUserData;
use mlua::UserData;
use mlua::Value;
use mlua::Lua;

/// Lua module that exposes a constructor for `EntityHandle`.
#[derive(Default)]
pub struct EntityModule;
register_lua_module!(EntityModule);

impl LuaModule for EntityModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
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

impl UserData for EntityHandle {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        // entity:get("Component")
        methods.add_method("get", |lua, this, comp_name: String| {
            let globals = lua.globals();
            let user_data: AnyUserData = globals.get("GameCtx")?;
            let ctx = user_data.borrow::<LuaGameCtx>()?;

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
        methods.add_method("set", |_lua, this, (comp_name, value): (String, Value)| {
            push_command(Box::new(SetComponentCmd { 
                entity: *this.entity, 
                comp_name, 
                value,
            }));
            Ok(())
        });

        // e:has("Component")
        // methods.add_method("has", |_, this, comp_name: String| {
        //     let (tx, rx) = mpsc::channel();
        //     push_command(Box::new(GetComponentCmd{ 
        //         entity: *this.entity, 
        //         comp_name, 
        //         responder: tx,
        //     }));
        //     match rx.recv() {
        //         Ok(Ok(_)) => Ok(true),
        //         _ => Ok(false),
        //     }
        // });

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