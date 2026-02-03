// game/src/scripting/lua_ctx.rs
use crate::game_state::GameState;
use engine_core::scripting::lua_constants::*;
use mlua::prelude::LuaResult;
use std::cell::RefCell;
use mlua::UserDataRef;
use mlua::UserData;
use std::rc::Rc;
use mlua::Lua;

/// The Lua‑exposed context that gives scripts access to the current `GameState`.
#[derive(Clone)]
pub struct LuaGameCtx {
    pub game_state: Rc<RefCell<GameState>>,
}

impl UserData for LuaGameCtx {}

impl LuaGameCtx {
    /// Registers this `LuaGameCtx` instance in the Lua global table.
    pub fn set_lua_game_ctx(self, lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();
        globals.set(LUA_GAME_CTX, self)?;
        Ok(())
    }

    /// Retrieves a borrowed reference to the stored `LuaGameCtx`.
    pub fn borrow_ctx<'lua>(lua: &'lua Lua) ->  LuaResult<UserDataRef<LuaGameCtx>> {
        let globals = lua.globals();
        let user_data: mlua::AnyUserData = globals.get(LUA_GAME_CTX)?;
        user_data.borrow::<LuaGameCtx>()
    }
}