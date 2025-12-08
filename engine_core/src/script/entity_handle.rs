// engine_core/src/script/entity_handle.rs
use mlua::Value;
use crate::ecs::component_registry::COMPONENTS;
use mlua::UserDataMethods;
use mlua::UserData;
use crate::ecs::world_ecs::WorldEcs;
use std::sync::Mutex;
use std::sync::Arc;
use crate::ecs::entity::Entity;

/// A thin wrapper that carries an Entity id and a reference to the whole world.
#[derive(Clone)]
pub struct EntityHandle {
    pub entity: Entity,
    pub world: Arc<Mutex<WorldEcs>>,
}

impl UserData for EntityHandle {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        // generic `has("Component")`
        methods.add_method("has", |_, this, comp_name: String| {
            let world = this.world.lock().unwrap();
            let reg = COMPONENTS.iter()
                .find(|r| r.type_name == comp_name)
                .ok_or_else(|| mlua::Error::RuntimeError(
                    format!("Component '{}' not known", comp_name)))?;
            Ok((reg.has)(&world, this.entity))
        });

        // generic `get("Component")`
        methods.add_method("get", |lua, this, comp_name: String| {
            let world = this.world.lock().unwrap();
            let reg = COMPONENTS.iter()
                .find(|r| r.type_name == comp_name)
                .ok_or_else(|| mlua::Error::RuntimeError(
                    format!("Component '{}' not known", comp_name)))?;

            if !(reg.has)(&world, this.entity) {
                return Ok(Value::Nil);
            }

            let boxed = (reg.clone)(&world, this.entity);
            let lua_val = (reg.to_lua)(&lua, &*boxed)?;
            Ok(lua_val)
        });

        // generic `set("Component", Value)`
        methods.add_method("set", |lua, this, (comp_name, tbl): (String, Value)| {
            let mut world = this.world.lock().unwrap();
            let reg = COMPONENTS.iter()
                .find(|r| r.type_name == comp_name)
                .ok_or_else(|| mlua::Error::RuntimeError(
                    format!("Component '{}' not known", comp_name)))?;

            // Convert the Lua table directly into a boxed component
            let boxed = (reg.from_lua)(lua, tbl)?;
            (reg.inserter)(&mut world, this.entity, boxed);
            Ok(())
        });

        // convenience: `entity.id` (read‑only)
        methods.add_method("id", |_, this, ()| Ok(this.entity.0));
    }
    
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {}
    
    fn register(registry: &mut mlua::UserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}