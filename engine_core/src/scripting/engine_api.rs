// engine_core/src/script/engine_api.rs
use mlua::Function;
use mlua::prelude::LuaResult;
use mlua::Value;
use mlua::Variadic;
use mlua::Lua;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// The type of a Rust callback that can be called from Lua.
type EngineFn = Arc<dyn Fn(&Lua, Variadic<Value>) -> LuaResult<Value> + Send + Sync>;

#[derive(Default)]
pub struct EngineApi {
    /// Map of function names to Rust callbacks.
    pub callbacks: Mutex<HashMap<String, EngineFn>>,
    /// Event listeners.
    pub listeners: Mutex<HashMap<String, Vec<Function>>>,
}

impl EngineApi {
    /// Register a new function.
    pub fn register<F>(&self, name: impl Into<String>, f: F)
    where
        F: Fn(&Lua, Variadic<Value>) -> LuaResult<Value> + Send + Sync + 'static,
    {
        let mut map = self.callbacks.lock().unwrap();
        map.insert(name.into(), Arc::new(f));
    }

    /// The implementation of `engine.call(name, …)` that Lua sees.
    pub fn lua_call<'lua>(&self, lua: &'lua Lua, args: Variadic<Value>) -> LuaResult<Value> {
        // The first argument must be the function name
        let mut iter = args.into_iter();
        let name = match iter.next() {
            Some(Value::String(s)) => s.to_str()?.to_owned(),
            _ => return Err(mlua::Error::FromLuaConversionError {
                from: "non-string",
                to: "String".to_owned(),
                message: Some("First argument to engine.call must be the function name.".into()),
            }),
        };

        // Look up the Rust callback
        let map = self.callbacks.lock().unwrap();
        let callback = map
            .get(&name)
            .ok_or_else(|| mlua::Error::RuntimeError(
                format!("Engine function '{name}' not registered.")
            ))?;

        // Pass the remaining arguments to the Rust callback
        callback(lua, Variadic::from_iter(iter))
    }
}