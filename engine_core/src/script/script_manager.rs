// engine_core/src/script/script_manager.rs
use crate::input::input_snapshot::InputSnapshot;
use crate::ecs::world_ecs::WorldEcs;
use std::sync::Mutex;
use mlua::Function;
use mlua::Variadic;
use crate::script::engine_api::EngineApi;
use mlua::Value;
use crate::game::game::Game;
use crate::script::script::ScriptId;
use crate::storage::path_utils::scripts_folder;
use crate::*;
use std::path::Path;
use std::fs;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use mlua::Lua;
use mlua::Table;
use serde::Deserialize;
use serde::Serialize;

/// A thin wrapper that owns the Lua VM and a reference to the world.
#[derive(Serialize, Deserialize, Default)]
pub struct ScriptManager {
    #[serde(skip)]
    pub lua: Lua,
    #[serde(skip)]
    engine_api: Arc<EngineApi>,
    /// Persistent map of all sprite is to their paths.
    pub script_id_to_path: HashMap<ScriptId, PathBuf>,
    #[serde(skip)]
    pub path_to_script_id: HashMap<PathBuf, ScriptId>,
    #[serde(skip)]
    /// Counter for script ids. Starts from 1.
    next_script_id: usize,
    /// Shared mutable reference to the game.
    pub game_name: String,
}

impl ScriptManager {
    /// Initializes a new script manager.
    pub async fn new(game_name: String) -> Self {
        let mut manager = Self {
            lua: Lua::new(),
            engine_api: Arc::new(EngineApi::default()),
            script_id_to_path: HashMap::new(),
            path_to_script_id: HashMap::new(),
            next_script_id: 1,
            game_name,
        };

        Self::register_all_modules(&mut manager);
        manager
    }

    pub fn load_table_from_id(&mut self, id: ScriptId) -> mlua::Result<Table> {
        let rel_path = self
            .script_id_to_path
            .get(&id)
            .ok_or_else(|| {
                mlua::Error::RuntimeError(format!("Unknown script id: {:?}.", id))
            })?;
        
        let abs_path = scripts_folder(&self.game_name).join(rel_path);

        let src = fs::read_to_string(abs_path)
            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

        let path_name = rel_path.display().to_string();

        let table: Table = self.lua.load(&src).set_name(path_name).eval()?;
        Ok(table)
    }

    /// Returns the id for `path`, loading it if necessary.
    pub fn get_or_load<P: AsRef<Path>>(&mut self, path: P) -> Option<ScriptId> {
        let p = path.as_ref();
        if p.to_string_lossy().trim().is_empty() {
            return None;
        }

        if let Some(&id) = self.path_to_script_id.get(p) {
            return Some(id);
        }
        
        match self.init_script(p) {
            Ok(id) => Some(id),
            Err(err) => {
                onscreen_error!("{}", err);
                None
            }
        }
    }

    /// Load and initialize a script from the scripts folder.
    /// Returns the `ScriptId` for the script.
    pub fn init_script(&mut self, rel_path: impl AsRef<Path>) -> Result<ScriptId, String> {
        let path = rel_path.as_ref().to_path_buf();

        if path.to_string_lossy().trim().is_empty() {
            // Guard against path being empty
            return Err("Empty script path.".into());
        }

        // Already loaded, reuse the same id
        if let Some(&id) = self.path_to_script_id.get(&path) {
            return Ok(id);
        }

        // Set and calculate the next texture id
        let id = ScriptId(self.next_script_id);
        self.restore_next_id();

        // Store everything
        self.path_to_script_id.insert(path.clone(), id);
        self.script_id_to_path.insert(id, path);

        return Ok(id);
    }

    /// Returns a path normalized relative to the game's scripts folder.
    pub fn normalize_path(&self, path: PathBuf) -> PathBuf {
        let scripts_dir = scripts_folder(&self.game_name);
        path.strip_prefix(&scripts_dir)
            .unwrap_or_else(|_| &path)
            .to_path_buf()
    }

    /// Initialize all scripts for the game.
    pub async fn init_manager(game: &mut Game) {
        // Calculate the next id from the existing map
        game.script_manager.restore_next_id();

        // Repopulate reverse map
        let scripts: Vec<(ScriptId, PathBuf)> = game.script_manager
            .script_id_to_path
            .iter()
            .map(|(id, path)| (*id, path.clone()))
            .collect();
        
        for (id, path) in scripts {
            game.script_manager.path_to_script_id.insert(path.clone(), id);
        }

        // Register all callbacks
        Self::register_all_modules(&mut game.script_manager);
    }

    /// Calculates the next script id 
    fn restore_next_id(&mut self) {
        let used: HashSet<_> = self.script_id_to_path
            .keys()
            .map(|sid| sid.0)
            .filter(|&id| id != 0)
            .collect();

        let mut candidate = 1usize;

        // Scan through until an unused id is found
        while used.contains(&candidate) {
            candidate += 1;
        }
        self.next_script_id = candidate;
    }

    /// Call this once after the `Lua` instance has been created.
    pub fn register_engine_module(&mut self) -> Result<(), mlua::Error> {
        let lua = &self.lua;

        // Build the module
        let engine_mod = lua.create_table()?;

        // engine.call(name, ...)
        let engine_api = self.engine_api.clone();
        let call_fn = lua.create_function(move |lua, args: Variadic<Value>| {
            engine_api.lua_call(lua, args)
        })?;
        engine_mod.set("call", call_fn)?;

        // Convenience wrappers (engine.log, engine.wait, …)
        let engine_api = self.engine_api.clone();
        for name in engine_api.callbacks.lock().unwrap().keys() {
            let fn_name = name.clone();
            let api = engine_api.clone();
            let wrapper = lua.create_function(move |lua, args: Variadic<Value>| {
                let mut full = vec![Value::String(lua.create_string(&fn_name)?)];
                full.extend_from_slice(&args);
                api.lua_call(lua, Variadic::from(full))
            })?;
            engine_mod.set(name.clone(), wrapper)?;
        }

        // engine.on(event, handler)
        let engine_api = self.engine_api.clone();
        let on_fn = lua.create_function(move |_, (event, handler): (String, Function)| {
            engine_api.listeners
                .lock()
                .unwrap()
                .entry(event)
                .or_default()
                .push(handler);
            Ok(())
        })?;
        engine_mod.set("on", on_fn)?;

        // engine.emit(event, …)
        let engine_api = self.engine_api.clone();
        let emit_fn = lua.create_function(move |_lua, (event, args): (String, Variadic<Value>)| {
            let map = engine_api.listeners.lock().unwrap();
            if let Some(callbacks) = map.get(&event) {
                for cb in callbacks {
                    if let Err(e) = cb.call::<()>(args.clone()) {
                        onscreen_error!("Lua listener error for event '{}': {}", event, e);
                    }
                }
            }
            Ok(())
        })?;
        engine_mod.set("emit", emit_fn)?;

        lua.register_module("engine", &engine_mod)?;
        Ok(())
    }

    /// Register the built‑in modules.
    fn register_modules(&mut self) {
        let engine_api = self.engine_api.clone();

        // Log: TODO: convert to module
        engine_api.register("log", |_, args| {
            let msg = match args.iter().next() {
                Some(Value::String(s)) => s.to_str()?.to_owned(),
                _ => return Err(mlua::Error::RuntimeError("log expects a string".into())),
            };
            // TODO: add multiple methods for levels
            onscreen_info!("[Lua] {}", msg);
            Ok(Value::Nil)
        });

        // let modules: Vec<Box<dyn script::lua_module::LuaModule>> = vec![
        //     Box::new(script::input_module::InputModule {
        //         snapshot: self.input_snapshot.clone(),
        //     }),
        //     Box::new(script::entity_module::EntityModule {
        //         world: self.world_arc.clone(),
        //     }),
        //     // add more modules here (physics, audio, UI, …)
        // ];

        // for m in modules {
        //     if let Err(e) = m.register(&self.lua) {
        //         onscreen_error!("Error registering lua module: {e}")
        //     }
        // }
    }

    fn register_all_modules(script_manager: &mut ScriptManager) {
        script_manager.register_modules();
        if let Err(e) = script_manager.register_engine_module() {
            onscreen_error!("Error registering engine module: {e}")
        }
    }
}