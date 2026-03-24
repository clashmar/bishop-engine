-- Auto-generated. Do not edit.
---@meta

--- Audio system module
---@class AudioApi
engine.audio = {}

--- Plays music by ID, looping until stopped. Stops any current track.
---@param id string Path relative to Resources/audio/ without extension
function engine.audio.play_music(id) end

--- Stops music immediately.
function engine.audio.stop_music() end

--- Fades music out over duration seconds, then stops.
---@param duration number Fade duration in seconds
function engine.audio.fade_music(duration) end

--- Plays a sound effect fire-and-forget.
---@param id string Path relative to Resources/audio/ without extension
function engine.audio.play_sfx(id) end

--- Pre-loads a sound into the cache to prevent stutter on first play.
---@param id string Path relative to Resources/audio/ without extension
function engine.audio.preload(id) end

--- Sets master volume (0.0–1.0).
---@param volume number
function engine.audio.set_master_volume(volume) end

--- Sets music group volume (0.0–1.0).
---@param volume number
function engine.audio.set_music_volume(volume) end

--- Sets SFX group volume (0.0–1.0).
---@param volume number
function engine.audio.set_sfx_volume(volume) end

