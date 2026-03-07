// game/src/scripting/modules/dialogue_module.rs
use crate::scripting::commands::dialogue_commands::SetLanguageCmd;
use crate::scripting::lua_ctx::LuaGameCtx;
use crate::game_global::push_command;
use engine_core::register_lua_api;
use engine_core::register_lua_module;
use engine_core::scripting::modules::lua_module::*;
use engine_core::scripting::lua_constants::*;
use mlua::prelude::LuaResult;
use mlua::Table;
use mlua::Lua;

/// Lua module that exposes the dialogue system API.
#[derive(Default)]
pub struct DialogueModule;
register_lua_module!(DialogueModule);

impl LuaModule for DialogueModule {
    fn register(&self, lua: &Lua) -> LuaResult<()> {
        let engine_tbl: Table = lua.globals().get(ENGINE)?;
        let dialogue_tbl = lua.create_table()?;

        let set_language_fn = lua.create_function(|_lua, lang: String| {
            push_command(Box::new(SetLanguageCmd { language: lang }));
            Ok(())
        })?;
        dialogue_tbl.set(SET_LANGUAGE, set_language_fn)?;

        let get_language_fn = lua.create_function(|lua, ()| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_instance = ctx.game_instance.borrow();
            let lang = game_instance.game.dialogue_manager.get_language().to_string();
            Ok(lang)
        })?;
        dialogue_tbl.set(GET_LANGUAGE, get_language_fn)?;

        let get_languages_fn = lua.create_function(|lua, ()| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_instance = ctx.game_instance.borrow();
            let langs: Vec<String> = game_instance
                .game
                .dialogue_manager
                .get_languages()
                .to_vec();
            let table = lua.create_table()?;
            for (i, lang) in langs.iter().enumerate() {
                table.set(i + 1, lang.clone())?;
            }
            Ok(table)
        })?;
        dialogue_tbl.set(GET_LANGUAGES, get_languages_fn)?;

        let get_config_fn = lua.create_function(|lua, ()| {
            let ctx = LuaGameCtx::borrow_ctx(lua)?;
            let game_instance = ctx.game_instance.borrow();
            let config = &game_instance.game.dialogue_manager.config;

            let table = lua.create_table()?;
            table.set("default_duration", config.default_duration)?;
            table.set("font_size", config.font_size)?;
            table.set("max_width", config.max_width)?;
            table.set("default_offset_y", config.default_offset_y)?;
            table.set("padding", config.padding)?;
            table.set("show_background", config.show_background)?;

            let color_tbl = lua.create_table()?;
            color_tbl.set(1, config.default_color[0])?;
            color_tbl.set(2, config.default_color[1])?;
            color_tbl.set(3, config.default_color[2])?;
            color_tbl.set(4, config.default_color[3])?;
            table.set("default_color", color_tbl)?;

            let bg_color_tbl = lua.create_table()?;
            bg_color_tbl.set(1, config.default_background_color[0])?;
            bg_color_tbl.set(2, config.default_background_color[1])?;
            bg_color_tbl.set(3, config.default_background_color[2])?;
            bg_color_tbl.set(4, config.default_background_color[3])?;
            table.set("default_background_color", bg_color_tbl)?;

            Ok(table)
        })?;
        dialogue_tbl.set(GET_CONFIG, get_config_fn)?;

        engine_tbl.set(DIALOGUE, dialogue_tbl)?;
        Ok(())
    }
}

register_lua_api!(DialogueModule, DIALOGUE_FILE);

impl LuaApi for DialogueModule {
    fn emit_api(&self, out: &mut LuaApiWriter) {
        out.line("--- Dialogue system module");
        out.line("---@class DialogueApi");
        out.line("engine.dialogue = {}");
        out.line("");

        out.line("--- Sets the current dialogue language.");
        out.line("---@param lang string The language code (e.g. \"en\", \"es\")");
        out.line("function engine.dialogue.set_language(lang) end");
        out.line("");

        out.line("--- Gets the current dialogue language.");
        out.line("---@return string");
        out.line("function engine.dialogue.get_language() end");
        out.line("");

        out.line("--- Gets a list of available languages.");
        out.line("---@return string[]");
        out.line("function engine.dialogue.get_languages() end");
        out.line("");

        out.line("--- Gets the current dialogue configuration.");
        out.line("---@return {default_duration: number, font_size: number, max_width: number, default_offset_y: number, padding: number, show_background: boolean, default_color: number[], default_background_color: number[]}");
        out.line("function engine.dialogue.get_config() end");
        out.line("");
    }
}
