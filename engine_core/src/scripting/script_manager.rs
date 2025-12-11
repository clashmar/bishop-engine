// engine_core/src/script/script_manager.rs
use crate::scripting::script::ScriptData;
use crate::scripting::script::ScriptField;
use mlua::prelude::LuaResult;
use mlua::Function;
use crate::scripting::engine_api::EngineApi;
use crate::game::game::Game;
use crate::scripting::script::ScriptId;
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

/// Manages access to scripts and holds the Lua VM instance.
#[derive(Serialize, Deserialize, Default)]
pub struct ScriptManager {
    #[serde(skip)]
    /// ???
    pub engine_api: Arc<EngineApi>,
    #[serde(skip)]
    /// Maps `ScriptId`'s to their loaded lua `Table`.
    pub tables: HashMap<ScriptId, Table>,
    #[serde(skip)]
    /// Maps ScriptId → optional update(dt) function TODO: is this necessary?
    pub update_fns: HashMap<ScriptId, Function>,
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
        let manager = Self {
            engine_api: Arc::new(EngineApi::default()),
            tables: HashMap::new(),
            update_fns: HashMap::new(),
            script_id_to_path: HashMap::new(),
            path_to_script_id: HashMap::new(),
            next_script_id: 1,
            game_name,
        };

        manager
    }

    /// Load the Lua table and store it in the manager
    pub fn load_script_table(&mut self, lua: &Lua, id: ScriptId) -> LuaResult<&Table> {
        if self.tables.contains_key(&id) {
            return Ok(self.tables.get(&id).unwrap());
        }

        let table = self.load_table_from_id(lua, id)?;

        if let Ok(update) = table.get::<_>("update") {
            self.update_fns.insert(id, update);
        }

        self.tables.insert(id, table);
        Ok(self.tables.get(&id).unwrap())
    }

    /// Get the table for a script.
    pub fn get_table(&self, id: ScriptId) -> Option<&Table> {
        self.tables.get(&id)
    }

    /// Get the update function for a script.
    pub fn get_update_fn(&self, id: ScriptId) -> Option<&Function> {
        self.update_fns.get(&id)
    }

    /// Sync ScriptData back into its Lua table.
    pub fn sync_to_lua(&self, lua: &Lua, id: ScriptId, data: &ScriptData) -> LuaResult<()> {
        let table = match self.tables.get(&id) {
            Some(t) => t,
            None => return Ok(()),
        };

        let public = table.get::<Option<Table>>("public")?
            .unwrap_or_else(|| table.clone());

        for (name, field) in &data.fields {
            match field {
                ScriptField::Bool(b) => public.set(name.clone(), *b)?,
                ScriptField::Int(i) => public.set(name.clone(), *i)?,
                ScriptField::Float(f) => public.set(name.clone(), *f)?,
                ScriptField::Text(s) => public.set(name.clone(), s.clone())?,
                ScriptField::Vec2(v) => {
                    let t = lua.create_table()?;
                    t.set(1, v[0])?;
                    t.set(2, v[1])?;
                    public.set(name.clone(), t)?;
                }
                ScriptField::Vec3(v) => {
                    let t = lua.create_table()?;
                    t.set(1, v[0])?;
                    t.set(2, v[1])?;
                    t.set(3, v[2])?;
                    public.set(name.clone(), t)?;
                }
            }
        }
        Ok(())
    }

    pub fn load_table_from_id(&mut self, lua: &Lua, id: ScriptId) -> LuaResult<Table> {
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

        let table: Table = lua.load(&src).set_name(path_name).eval()?;
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

    pub fn reload(&mut self, lua: &Lua, id: ScriptId) -> LuaResult<&Table> {
        self.tables.remove(&id);
        self.update_fns.remove(&id);
        self.load_script_table(lua, id)
    }

    pub fn unload(&mut self, id: ScriptId) {
        self.tables.remove(&id);
        self.update_fns.remove(&id);
        self.script_id_to_path.remove(&id);
        self.path_to_script_id.retain(|_, &mut v| v != id);
    }
}