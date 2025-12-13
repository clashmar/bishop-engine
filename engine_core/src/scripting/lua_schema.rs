// engine_core/src/scripting/lua_schema.rs

/// Trait that provides a **static** description of a component’s Lua shape.
pub trait LuaSchema {
    /// `(field_name, lua_type)` pairs.  The slice is `'static` so it can be stored
    /// directly in `ComponentRegistry`.
    const SCHEMA: &'static [(&'static str, &'static str)];
}

/// Helper that maps a Rust type name (as a string) to a Lua primitive.
/// Extend this match if you have custom types you want to treat specially.
fn rust_type_to_lua(ty_name: &str) -> &'static str {
    match ty_name {
        // numeric primitives – all become Lua “number”
        "f32" | "f64"
        | "i8" | "i16" | "i32" | "i64"
        | "u8" | "u16" | "u32" | "u64"
        | "usize" | "isize" => "number",

        // integers only – you could return "integer" if you prefer
        // "i32" => "integer",

        "bool" => "boolean",
        "String" => "string",
        "&str" => "string",

        // Anything else is treated as a generic Lua table / userdata
        _ => "table",
    }
}

macro_rules! field_array {
    ($ty:ty) => {{
        // Create a temporary value of the type – it must be `Default`.
        // All your components already derive `Default`, otherwise add it.
        const TMP: $ty = <$ty>::default();

        // Collect the static data.
        // NOTE: we have to do this in a const context, so we use a
        // `match` on a slice of `FieldInfo` that we turn into a literal.
        const INFO: &[crate::ecs::reflect_field::FieldInfo<'static>] = {
            // SAFETY: `fields` only reads `self`, never mutates.
            // The temporary is `const`, so this is fine.
            unsafe { &*(&TMP as *const _ as *const _) }.fields()
        };

        // Turn the `FieldInfo` slice into a literal array.
        // The macro expands to something like:
        //   &[("x", "number"), ("y", "number")]
        const ARR: &[(&'static str, &'static str)] = &{
            let mut out = [("", ""); INFO.len()];
            let mut i = 0;
            while i < INFO.len() {
                let name = INFO[i].name;
                // The type name is stored as a string slice inside `FieldInfo`.
                // We map it to a Lua primitive.
                let lua_ty = rust_type_to_lua(INFO[i].ty_name);
                out[i] = (name, lua_ty);
                i += 1;
            }
            out
        };
        ARR
    }};
}

impl<T> LuaSchema for T
where
    T: crate::ecs::reflect_field::Reflect,
{
    const SCHEMA: &'static [(&'static str, &'static str)] = {
        // We cannot create a true `const` from a `Vec`, but we can cheat
        // with a macro that expands the fields into a literal array.
        // The `field_array!` macro is defined a few lines below.
        field_array!(T)
    };
}