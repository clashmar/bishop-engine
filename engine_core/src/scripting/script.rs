// engine_core/src/script/script.rs
use crate::scripting::script_manager::ScriptManager;
use std::collections::HashMap;
use mlua::prelude::LuaResult;
use crate::ecs_component;
use serde::Deserialize;
use serde::Serialize;
use mlua::Table;
use mlua::Value;

/// Opaque handle that the script manager gives out. Default/Unset is 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ScriptId(pub usize);

/// One field that can be edited in the inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptField {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
}

/// The script data that the editor shows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptData {
    pub fields: HashMap<String, ScriptField>,
}

/// The script component that lives on an entity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Script {
    /// Id stored by the script manager.
    pub script_id: ScriptId,
    /// The public fields that the inspector can edit.
    pub data: ScriptData,
}

ecs_component!(Script);

impl Script {
    /// Loads the table from ScriptManager and updates ScriptData
    pub fn load(&mut self, script_manager: &mut ScriptManager) -> LuaResult<()> {
        if self.script_id.0 == 0 {
            // Script hasn't been set yet
            self.data.fields.clear();
            return Ok(());
        }

        let table = script_manager.load_script_table(self.script_id)?;

        // Determine the public fields table
        let public: Table = match table.get::<Option<Table>>("public")? {
            Some(t) => t,
            None => table.clone(),
        };

        let mut fields = HashMap::new();

        for pair in public.pairs::<String, Value>() {
            let (name, value) = pair?;
            let field = match value {
                mlua::Value::Boolean(b) => ScriptField::Bool(b),
                mlua::Value::Integer(i) => ScriptField::Int(i),
                mlua::Value::Number(n) => ScriptField::Float(n),
                mlua::Value::String(s) => ScriptField::Text(s.to_str()?.to_string()),
                mlua::Value::Table(t) => {
                    // Try Vec2
                    if let Ok(x) = t.get::<f64>(1) {
                        if let Ok(y) = t.get::<f64>(2) {
                            if let Ok(z) = t.get::<f64>(3) {
                                ScriptField::Vec3([x as f32, y as f32, z as f32])
                            } else {
                                ScriptField::Vec2([x as f32, y as f32])
                            }
                        } else {
                            continue; // skip unsupported table
                        }
                    } else {
                        continue; // skip unsupported table
                    }
                }
                _ => continue, // ignore function, userdata, etc.
            };
            fields.insert(name, field);
        }

        // Remove any stale fields
        self.data.fields.retain(|name, _| fields.contains_key(name));
        // Add or update fields
        for (name, field) in fields {
            self.data.fields.entry(name).or_insert(field);
        }

        // Sync current values back to Lua
        self.sync_to_lua(script_manager)?;

        Ok(())
    }

     /// Sync the current ScriptData back to Lua table.
    pub fn sync_to_lua(&self, script_manager: &ScriptManager) -> LuaResult<()> {
        script_manager.sync_to_lua(self.script_id, &self.data)
    }
}


