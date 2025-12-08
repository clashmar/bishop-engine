// engine_core/src/script/script.rs
use mlua::Function;
use mlua::prelude::LuaValue;
use mlua::Table;
use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use crate::ecs_component;
use crate::script::script_manager::ScriptManager;

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
    /// The Lua table created at load time.
    #[serde(skip)]
    pub table: Option<Table>,
    #[serde(skip)]
    pub update_fn: Option<Function>, 
}

ecs_component!(Script);

impl Script {
    /// Loads the script file, evaluates it, and fill updates data fields.
    pub fn load(
        &mut self, 
        script_manager: &mut ScriptManager
    ) -> mlua::Result<()> {
        if self.table.is_some() {
            return Ok(());
        }

        let table = script_manager.load_table_from_id(self.script_id)?;

        // Look for `update(dt)`
        if let Ok(update) = table.get::<Function>("update") {
            self.update_fn = Some(update);
        }
        
        let public = table
            .get::<Option<Table>>("public")?
            .unwrap_or_else(|| table.clone());

        self.table = Some(table);
        
        // Convert each Lua value into a ScriptField
        let mut fields = HashMap::new();

        for pair in public.pairs::<String, LuaValue>() {
            let (name, value) = pair?;
            let field = match value {
                LuaValue::Boolean(b) => ScriptField::Bool(b),
                LuaValue::Integer(i) => ScriptField::Int(i),
                LuaValue::Number(n) => ScriptField::Float(n),
                LuaValue::String(s) => ScriptField::Text(s.to_str()?.to_owned()),
                LuaValue::Table(t) => {
                    // Try to recognise a Vec2 or Vec3.
                    if let Ok(x) = t.get::<f64>(1) {
                        if let Ok(y) = t.get::<f64>(2) {
                            if let Ok(z) = t.get::<f64>(3) {
                                ScriptField::Vec3([x as f32, y as f32, z as f32])
                            } else {
                                ScriptField::Vec2([x as f32, y as f32])
                            }
                        } else {
                            // Ignore unsupported tables
                            continue; 
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue, // functions, userdata, etc. are ignored
            };
            fields.insert(name, field);
        }

        // Remove stale fields
        self.data.fields.retain(|name, _| fields.contains_key(name));

        // Only update fields that didn't previously exist
        for (name, field) in fields {
            self.data.fields.entry(name).or_insert(field);
        }

        // Make sure any stored values are written back to the table
        self.sync_to_lua(script_manager)?;
        Ok(())
    }

    /// Write the current data back into the Lua table.
    pub fn sync_to_lua(&self, script_manager: &mut ScriptManager) -> mlua::Result<()> {
        let lua = &script_manager.lua;

        let table = match &self.table {
            Some(t) => t,
            None => return Ok(()),
        };

        // Put everything under public if the sub‑table exists
        let public = table
            .get::<Option<Table>>("public")?
            .unwrap_or_else(|| table.clone());

        for (name, field) in &self.data.fields {
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
}


