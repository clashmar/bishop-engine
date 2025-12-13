// lua_api_gen/src/main.rs
use engine_core::scripting::modules::lua_module::LuaApiRegistry;
use engine_core::scripting::modules::lua_module::LuaApiWriter;
use std::path::PathBuf;
use std::env;
use std::fs;
use game_lib as _;

fn main() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../editor/scripts/_engine");

    fs::create_dir_all(&out_dir).unwrap();
    println!("create dirs");

    for reg in inventory::iter::<LuaApiRegistry> {
        let module = (reg.ctor)();
        let mut writer = LuaApiWriter::new();
        module.emit_api(&mut writer);
        let filename = reg.name.to_lowercase();
        let path = out_dir.join(format!("{filename}.lua"));
        fs::write(&path, writer.buf).unwrap();
        println!("Written to: {}", path.display());
    }
}
