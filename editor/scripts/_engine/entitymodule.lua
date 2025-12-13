-- Auto-generated. Do not edit.
---@meta

---@class Entity
---@field id integer
local Entity = {}

---@param component string
---@see ComponentId
---@return table
function Entity:get(component) end

---@param component string
---@see ComponentId
---@param value table
function Entity:set(component, value) end

---@param component string
---@see ComponentId
---@return boolean
function Entity:has(component) end

---@param ... string
---@see ComponentId
---@return boolean
function Entity:has_any(...) end

---@param ... string
---@see ComponentId
---@return boolean
function Entity:has_all(...) end

return Entity
