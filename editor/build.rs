// editor/build.rs
use engine_core::ecs::component_registry::COMPONENTS;
use engine_core::input::input_table::*;
use std::collections::HashSet;
use std::path::PathBuf;
use std::env;
use std::fs;

fn main() -> std::io::Result<()> {
    generate_lua_script();
    generate_lua_components();
    generate_lua_input();

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileVersion", "1.0.0.0")
            .set_icon("windows/Icon.ico")
            .set("FileDescription", "Bishop Engine: a cross platform 2dD editor.")
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

fn generate_lua_components() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("scripts")
        .join("_engine");

    fs::create_dir_all(&out_dir).expect("cannot create _engine folder");

    let mut lua = String::from(
        "-- Auto-generated. Do not edit.\n\
        ---@meta\n\
        ---@alias vec2 { x: number, y: number }\n\
        ---@alias vec3 { x: number, y: number, z: number }\n\n"
    );
    
    // TODO: convert to enum
    // Generate class definitions for each component with their schema
    for reg in COMPONENTS.iter() {
        let schema = (reg.lua_schema)();
        
        // Always generate a class definition, even for empty components
        lua.push_str(&format!("---@class {}\n", reg.type_name));
        
        if schema.is_empty() {
            // For marker/unit structs, add a comment
            lua.push_str("--- Marker component\n");
        } else {
            // Add field annotations from the schema
            for (field_name, field_type) in schema {
                lua.push_str(&format!("---@field {} {}\n", field_name, field_type));
            }
        }
        
        lua.push_str("\n");
    }
    
    // Generate the ComponentId class with all component names
    lua.push_str("---@class ComponentId\n");
    for reg in COMPONENTS.iter() {
        lua.push_str(&format!("---@field {} string\n", reg.type_name));
    }
    lua.push_str("\n");

    lua.push_str("local C = {}\n\n");

    // Fill table assignments
    for reg in COMPONENTS.iter() {
        lua.push_str(&format!("C.{} = \"{}\"\n", reg.type_name, reg.type_name));
    }

    lua.push_str("\nreturn C\n");

    let target = out_dir.join("components.lua");
    fs::write(&target, lua).expect("Cannot write components.lua");
    println!("cargo:warning=generated {}", target.display());
}

fn generate_lua_input() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("scripts")
        .join("_engine");

    fs::create_dir_all(&out_dir).expect("cannot create _engine folder");

    let mut lua = String::from(
        "-- Auto-generated. Do not edit.\n\
        ---@meta\n\n"
    );

    // Enum definition
    lua.push_str("---@enum Input\nlocal Input = {\n");

    // Avoids duplicates
    let mut seen = HashSet::new();
    
    // Keyboard
    for &(name, _code) in KEY_TABLE.iter() {
        // Nmae is the literal string that should be used at runtime
        if seen.insert(name) {
            let key = lua_key_name(name);
            lua.push_str(&format!("    {} = \"{}\",\n", key, name));
        }
    }

    // Mouse
    for &(name, _code) in MOUSE_TABLE.iter() {
        if seen.insert(name) {
            let key = lua_key_name(name);
            lua.push_str(&format!("    {} = \"{}\",\n", key, name));
        }
    }
    
    lua.push_str("}\n");
    
    // Return the enum table
    lua.push_str("\nreturn Input\n");

    // Write the file
    let target = out_dir.join("input.lua");
    fs::write(&target, lua).expect("Cannot write input.lua");
    println!("cargo:warning=generated {}", target.display());
}

fn generate_lua_script() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("scripts")
        .join("_engine");

    fs::create_dir_all(&out_dir).expect("cannot create _engine folder");

    let lua = String::from(
        "-- Auto-generated. Do not edit.\n\
        ---@meta\n\
        ---@class ScriptDef\n\
        ---@field public table\n\
        ---@field update fun(self: Script, dt: number)\n\
        ---@field init fun(self: Script)\n\
        ---@class Script : ScriptDef\n\
        ---@field entity Entity\n\
        local Script = {}\n\
        return Script\n"
    );

    let target = out_dir.join("script.lua");
    fs::write(&target, lua).expect("Cannot write script.lua");
    println!("cargo:warning=generated {}", target.display());
}