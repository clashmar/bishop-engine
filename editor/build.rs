use engine_core::ecs::component_registry::COMPONENTS;
use std::path::PathBuf;
use std::env;
use std::fs;

fn main() -> std::io::Result<()> {
    generate_lua_source();

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileVersion", "1.0.0.0")
            .set_icon("windows/Icon.ico")
            .set("FileDescription", "Bishop Engine: a cross platform 2D editor.")
            .set("ProductVersion", "1.0.0.0")
            .set("ProductName", "Bishop Engine")
            .set("OriginalFilename", "Bishop.exe")
            .set("LegalCopyright", "© 2025 Clashmar")
            .set("LegalTrademark", "Bishop Engine™")
            .set("CompanyName", "Clashmar Ltd.")
            .set("Comments", "Lightweight 2D Editor")
            .set("InternalName", "Bishop Engine")
            .set_version_info(winres::VersionInfo::FILEVERSION, 0x0001000000000000)
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000);

        res.compile()?;
    }
    Ok(())
}

fn generate_lua_source() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("scripts")
        .join("_engine");

    // Make sure the directory exists.
    fs::create_dir_all(&out_dir).expect("cannot create _engine folder");

    // Generate the lua source
    let mut lua = String::from(
        "-- This file is auto-generated. Do not edit manually.\n\
         ---@meta\n\
         ---@class ComponentId\n\
         local Component = {\n",
    );
    
    for reg in COMPONENTS.iter() {
        lua.push_str(&format!("    {} = {},\n", reg.type_name, reg.id));
    }
    
    lua.push_str("}\nreturn Component\n");

    // Write the file.
    let target = out_dir.join("components.lua");
    fs::write(&target, lua).expect("Cannot write components.lua");
    println!("cargo:warning=generated {}", target.display());
}