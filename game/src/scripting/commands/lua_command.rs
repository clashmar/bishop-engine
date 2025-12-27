// game/src/scripting/commands/lua_command.rs
use crate::engine::Engine;
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::scripting::script::Script;
use engine_core::ecs::entity::Entity;
use mlua::MultiValue;
use mlua::Function;
use engine_core::*;
use mlua::Value;

/// All mutating Lua actions implement this.
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

/// Calls a function on an entity.
pub struct CallEntityFnCmd {
    pub entity: Entity,
    pub fn_name: String,
    pub args: Vec<Value>,
}

// TODO: use this for updates?
impl LuaCommand for CallEntityFnCmd {
    fn execute(&mut self, engine: &mut Engine) {
        let game_state = engine.game_state.borrow();
        let world = &game_state.game.current_world().world_ecs;

        let script = match world.get::<Script>(self.entity) {
            Some(s) => s,
            None => return,
        };
        
        let instance = match game_state
        .game
        .script_manager
        .instances
        .get(&(self.entity, script.script_id)) {
            Some(t) => t,
            None => return,
        };
        
        let Ok(func) = instance.get::<Function>(&*self.fn_name) else {
            return;
        };

        let handle = Value::Table(instance.clone());

        let mut call_args = Vec::with_capacity(self.args.len() + 1);
        call_args.push(handle);
        call_args.extend(self.args.clone());

        if let Err(e) = func.call::<()>(MultiValue::from_vec(call_args)) {
            onscreen_error!("Lua call failed: {}", e);
        }
    }
}
