
use mlua::prelude::LuaResult;
use mlua::Lua;
use crate::script::lua_module::LuaModule;
use crate::input::input_snapshot::InputSnapshot;
use std::sync::Mutex;
use std::sync::Arc;

pub struct InputModule {
    pub snapshot: Arc<Mutex<InputSnapshot>>,
}

impl LuaModule for InputModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        // engine.input.is_down("key")
        let snap = self.snapshot.clone();
        let is_down = lua.create_function(move |_, key: String| {
            let snap = snap.lock().unwrap();
            Ok(snap.down.get(key.as_str()).copied().unwrap_or(false))
        })?;
        // engine.input.pressed("key")
        let snap = self.snapshot.clone();
        let pressed = lua.create_function(move |_, key: String| {
            let snap = snap.lock().unwrap();
            Ok(snap.pressed.get(key.as_str()).copied().unwrap_or(false))
        })?;
        // engine.input.released("key")
        let snap = self.snapshot.clone();
        let released = lua.create_function(move |_, key: String| {
            let snap = snap.lock().unwrap();
            Ok(snap.released.get(key.as_str()).copied().unwrap_or(false))
        })?;

        // Build a sub‑module `engine.input`
        let engine_mod = lua.globals().get::<mlua::Table>("engine")?;
        let input_mod = lua.create_table()?;
        input_mod.set("is_down", is_down)?;
        input_mod.set("pressed", pressed)?;
        input_mod.set("released", released)?;
        engine_mod.set("input", input_mod)?;
        Ok(())
    }
}