-- Auto-generated. Do not edit.
---@meta

--- Dialogue system module
---@class DialogueApi
engine.dialogue = {}

--- Sets the current dialogue language.
---@param lang string The language code (e.g. "en", "es")
function engine.dialogue.set_language(lang) end

--- Gets the current dialogue language.
---@return string
function engine.dialogue.get_language() end

--- Gets a list of available languages.
---@return string[]
function engine.dialogue.get_languages() end

--- Gets the current dialogue configuration.
---@return {default_duration: number, font_size: number, max_width: number, default_offset_y: number, padding: number, show_background: boolean, default_color: number[], default_background_color: number[]}
function engine.dialogue.get_config() end

