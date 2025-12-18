// engine_core/src/scripting/modules/lua_module.rs
use mlua::prelude::LuaResult;
use std::fmt::Write;
use mlua::Lua;

/// Every system that wants to expose Lua functions implements this.
pub trait LuaModule {
    /// Registers the module’s functions, types and globals with the given Lua state.
    fn register(&self, lua: &Lua) -> LuaResult<()>;
}

/// Registry that the inventory crate will collect.                  
pub struct LuaModuleRegistry {
    /// Called once for every module during start‑up.
    pub ctor: fn() -> Box<dyn LuaModule>,
}

// Collect all modules into a slice that lives for the whole program.
inventory::collect!(LuaModuleRegistry);

/// Trait which ensures lua api is implemented for a module.
pub trait LuaApi {
    /// Emit Lua signatures.
    fn emit_api(&self, out: &mut LuaApiWriter);
}

/// Writes the lua api for a module.
pub struct LuaApiWriter {
    pub buf: String,
}

impl LuaApiWriter {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn line(&mut self, s: &str) {
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    pub fn write(&mut self, args: std::fmt::Arguments) {
        let _ = self.buf.write_fmt(args);
    }
}

pub struct LuaApiRegistry {
    pub name: &'static str,
    pub ctor: fn() -> Box<dyn LuaApi>,
}

inventory::collect!(LuaApiRegistry);

#[macro_export]
macro_rules! register_lua_api {
    ($ty:ty) => {
        inventory::submit! {
            $crate::scripting::modules::lua_module::LuaApiRegistry {
                name: stringify!($ty),
                ctor: || Box::new(<$ty>::default()),
            }
        }
    };
}

#[macro_export]
macro_rules! register_lua_module {
    ($ty:ty) => {
        inventory::submit! {
            $crate::scripting::modules::lua_module::LuaModuleRegistry {
                ctor: || {
                    // Enforces each module to implement its api generation.
                    fn _assert<T: $crate::scripting::modules::lua_module::LuaExposedModule>() {}
                    _assert::<$ty>();
                    Box::new(<$ty>::default())
                },
            }
        }
    };
}

pub trait LuaExposedModule: LuaModule + LuaApi {}

impl<T> LuaExposedModule for T
where
    T: LuaModule + LuaApi
{}

/// Writes the module api to a .lua file.
pub fn generate_lua_api(out_dir: &std::path::Path) {
    std::fs::create_dir_all(out_dir).unwrap();

    for reg in inventory::iter::<LuaApiRegistry> {
        let module = (reg.ctor)();
        let mut writer = LuaApiWriter::new();
        module.emit_api(&mut writer);

        let name = std::any::type_name_of_val(&*module)
            .rsplit("::")
            .next()
            .unwrap()
            .to_lowercase();

        std::fs::write(out_dir.join(format!("{name}.lua")), writer.buf).unwrap();
    }
}