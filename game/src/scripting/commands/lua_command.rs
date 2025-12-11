// game/src/scripting/commands/lua_command.rs
use crate::game_state::GameState;
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::ecs::entity::Entity;
use std::sync::mpsc::Sender;
use mlua::Function;
use engine_core::*;
use mlua::Value;

/// All Lua actions implement this.
pub trait LuaCommand: Send {
    /// Execute the command, mutating the supplied `GameState`.
    fn execute(&mut self, game_state: &mut GameState);
}

/// Set a component on an entity.
pub struct SetComponentCmd {
    pub entity: usize,
    pub comp_name: String,
    pub value: Value,
}

impl LuaCommand for SetComponentCmd {
    fn execute(&mut self, game_state: &mut GameState) {
        if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == self.comp_name) {
            if let Ok(boxed) = (reg.from_lua)(&game_state.game.script_manager.lua, self.value.clone()) {
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

/// Query a component and send the result back through a channel.
pub struct GetComponentCmd {
    pub entity: usize,
    pub comp_name: String,
    pub responder: Sender<mlua::Result<Value>>,
}

impl LuaCommand for GetComponentCmd {
    fn execute(&mut self, game_state: &mut GameState) {
        let result = if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == self.comp_name) {
            let world = &mut game_state.game.current_world_mut().world_ecs;
            if (reg.has)(world, Entity(self.entity)) {
                let boxed = (reg.clone)(world, Entity(self.entity));
                (reg.to_lua)(&game_state.game.script_manager.lua, &*boxed)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {} has no {} component",
                    self.entity, self.comp_name
                )))
            }
        } else {
            Err(mlua::Error::RuntimeError(format!(
                "Component '{}' not known", 
                self.comp_name,
            )))
        };
        let _ = self.responder.send(result);
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
    fn execute(&mut self, game_state: &mut GameState) {
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

/// Catch‑all for ad‑hoc closures.
pub struct CustomCmd {
    pub cb: Option<Box<dyn FnOnce(&mut WorldEcs) + Send>>,
}

impl LuaCommand for CustomCmd {
    fn execute(&mut self, game_state: &mut GameState) {
        if let Some(cb) = self.cb.take() {
            cb(&mut game_state
                .game
                .current_world_mut()
                .world_ecs);
        } else {
            onscreen_error!("Unexpected error executing custom command.")
        }
    }
}