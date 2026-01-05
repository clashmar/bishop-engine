-- Auto-generated. Do not edit.
---@meta

--- Call a method on a global entity script
--- @param name string The name of the global entity
--- @param method string The method name to call
--- @param ... any Additional arguments to pass to the method
--- @return any Returns whatever the method returns
function engine.call(name, method, ...) end

--- Register an event handler
--- @param event string The name of the event to listen for
--- @param handler function The Lua function that will be called
--- @return nil
function engine.on(event, handler) end

--- Emit an event to all registered handlers
--- @param event string The name of the event to emit
--- @param ... any Arguments that will be passed to each handler
--- @return nil
function engine.emit(event, ...) end

---@param input string
function engine.input.is_down(input) end

---@param input string
function engine.input.pressed(input) end

---@param input string
function engine.input.released(input) end

---@param msg string
function engine.log.info(msg) end

---@param msg string
function engine.log.warn(msg) end

---@param msg string
function engine.log.error(msg) end

---@param msg string
function engine.log.debug(msg) end

