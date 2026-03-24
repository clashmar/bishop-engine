local GameManager = require("game_manager")
engine.game_manager = GameManager

local input = require("_engine.input")

engine.on("slider:master_volume", function(value)
    engine.audio.set_master_volume(value)
end)
engine.on("slider:music_volume", function(value)
    engine.audio.set_music_volume(value)
end)
engine.on("slider:sfx_volume", function(value)
    engine.audio.set_sfx_volume(value)
end)

engine.update = function(dt)
    if engine.input.pressed(input.M) then
        engine.menu.open("start")
    end
end