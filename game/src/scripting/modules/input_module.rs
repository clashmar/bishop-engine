// game/src/scripting/modules/input_module.rs
use crate::game_global::get_input_snapshot;
use crate::input::input_snapshot::InputSnapshot;
use engine_core::scripting::modules::lua_module::LuaModule;
use engine_core::scripting::lua_constants::*;
use engine_core::register_lua_module;
use std::collections::HashMap;
use mlua::prelude::LuaResult;
use mlua::Function;
use mlua::Table;
use mlua::Lua;

pub const INPUT_IS_DOWN: &str = "is_down";
pub const INPUT_PRESSED: &str = "pressed";
pub const INPUT_RELEASED: &str = "released";

/// Lua module that exposes the current input snapshot.
#[derive(Default, Clone)]
pub struct InputModule;
register_lua_module!(InputModule);

impl LuaModule for InputModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        let is_down_fn = make_snapshot_query_fn(lua, |snap| &snap.down)?;
        let pressed_fn = make_snapshot_query_fn(lua, |snap| &snap.pressed)?;
        let released_fn = make_snapshot_query_fn(lua, |snap| &snap.released)?;

        // Assemble the `engine.input` table
        let engine_tbl: Table = lua.globals().get(ENGINE)?;
        let input_tbl = lua.create_table()?;
        input_tbl.set(INPUT_IS_DOWN, is_down_fn)?;
        input_tbl.set(INPUT_PRESSED, pressed_fn)?;
        input_tbl.set(INPUT_RELEASED, released_fn)?;

        // Attach the sub‑module to the already existing global `engine` table
        engine_tbl.set(INPUT, input_tbl)?;

        Ok(())
    }
}

/// Build a Lua function that queries a current `InputSnapshot`.
pub fn make_snapshot_query_fn<'lua, Sel>(
    lua: &'lua Lua,
    map_selector: Sel,
) -> LuaResult<Function>
where
    Sel: Fn(&InputSnapshot) -> &HashMap<&'static str, bool> + Copy + Send + 'static,
{
    lua.create_function(move |_lua, key: String| {
        let mut snapshot = get_input_snapshot();
        snapshot.capture_input_state();
        
        let value = map_selector(&snapshot)
            .get(key.as_str())
            .copied()
            .unwrap_or(false);

        Ok(value)
    })
}