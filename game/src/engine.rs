use engine_core::scripting::script_manager::ScriptManager;
// game/src/engine.rs
use mlua::UserData;
use mlua::Lua;
use crate::game_state::GameState;
use std::cell::RefCell;
use std::rc::Rc;

struct Engine {
    game_state: Rc<RefCell<GameState>>,
    lua: Lua,
}

#[derive(Clone)]
pub struct LuaGameCtx {
    pub game_state: Rc<RefCell<GameState>>,
}

impl UserData for LuaGameCtx {}

impl LuaGameCtx {
    pub fn set_lua_game_ctx(self, script_manager: &ScriptManager) -> mlua::Result<()> {
        let globals = script_manager.lua.globals();
        globals.set("GameCtx", self)?;
        Ok(())
    }
}

