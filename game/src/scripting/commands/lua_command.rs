// game/src/scripting/commands/lua_command.rs
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::ecs::entity::Entity;
use std::sync::mpsc::Sender;
use crate::engine::Engine;
use mlua::Function;
use engine_core::*;
use mlua::Value;

/// All Lua actions implement this.
pub trait LuaCommand {
    /// Execute the command, mutating the supplied `GameState`.
    fn execute(&mut self, engine: &mut Engine);
}

/// Set a component on an entity.
pub struct SetComponentCmd {
    pub entity: usize,
    pub comp_name: String,
    pub value: Value,
}

impl LuaCommand for SetComponentCmd {
    fn execute(&mut self, engine: &mut Engine) {
        let mut game_state = engine.game_state.borrow_mut();
        if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == self.comp_name) {
            if let Ok(boxed) = (reg.from_lua)(&engine.lua, self.value.clone()) {
                (reg.inserter)(
                    &mut game_state.game.current_world_mut().world_ecs,
                    Entity(self.entity),
                    boxed,
                );
            } else {
                onscreen_error!("Failed to convert value for component '{}'", self.comp_name);
            }
        } else {
            onscreen_error!("Unknown component '{}'", self.comp_name);
        }
    }
}

/// Call a method on a global module.
pub struct CallGlobalCmd {
    pub name: String,
    pub method: String,
    pub args: Vec<Value>,
    pub responder: Sender<mlua::Result<Value>>,
}

impl LuaCommand for CallGlobalCmd {
    fn execute(&mut self, engine: &mut Engine) {
        let game_state = engine.game_state.borrow_mut();
        let result = game_state
            .global_modules
            .borrow()
            .get(&self.name)
            .cloned()
            .ok_or_else(|| mlua::Error::RuntimeError(format!("global '{}' not found", self.name)))
            .and_then(|val| match val {
                Value::Table(tbl) => {
                    let func: Function = tbl.get(&*self.method).unwrap();
                    func.call::<_>(self.args.clone())
                }
                _ => Err(mlua::Error::RuntimeError(format!(
                    "global '{}' is not a table",
                    self.name
                ))),
            });
        let _ = self.responder.send(result);
    }
}