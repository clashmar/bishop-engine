-- Auto-generated. Do not edit.
---@meta

---@class Entity
---@field id integer
local Entity = {}

-- Component getters
---@overload fun(self: Entity, component: "Sprite"): Sprite
---@overload fun(self: Entity, component: "Animation"): Animation
---@overload fun(self: Entity, component: "Interactable"): Interactable
---@overload fun(self: Entity, component: "Script"): Script
---@overload fun(self: Entity, component: "Light"): Light
---@overload fun(self: Entity, component: "Glow"): Glow
---@overload fun(self: Entity, component: "CurrentFrame"): CurrentFrame
---@overload fun(self: Entity, component: "Damage"): Damage
---@overload fun(self: Entity, component: "Solid"): Solid
---@overload fun(self: Entity, component: "Walkable"): Walkable
---@overload fun(self: Entity, component: "Kinematic"): Kinematic
---@overload fun(self: Entity, component: "PhysicsBody"): PhysicsBody
---@overload fun(self: Entity, component: "Collider"): Collider
---@overload fun(self: Entity, component: "Grounded"): Grounded
---@overload fun(self: Entity, component: "Velocity"): Velocity
---@overload fun(self: Entity, component: "Player"): Player
---@overload fun(self: Entity, component: "CurrentRoom"): CurrentRoom
---@overload fun(self: Entity, component: "Layer"): Layer
---@overload fun(self: Entity, component: "Position"): Position
---@overload fun(self: Entity, component: "RoomCamera"): RoomCamera
---@param component string
---@return table|nil
function Entity:get(component) end

-- Generic set method
---@param component string
---@see ComponentId
---@param value table
function Entity:set(component, value) end

-- Typed component setters
---@param self Entity
---@param v Sprite
function Entity:set_sprite(v) end

---@param self Entity
---@param v Animation
function Entity:set_animation(v) end

---@param self Entity
---@param v Interactable
function Entity:set_interactable(v) end

---@param self Entity
---@param v Script
function Entity:set_script(v) end

---@param self Entity
---@param v Light
function Entity:set_light(v) end

---@param self Entity
---@param v Glow
function Entity:set_glow(v) end

---@param self Entity
---@param v CurrentFrame
function Entity:set_current_frame(v) end

---@param self Entity
---@param v Damage
function Entity:set_damage(v) end

---@param self Entity
---@param v Solid
function Entity:set_solid(v) end

---@param self Entity
---@param v Walkable
function Entity:set_walkable(v) end

---@param self Entity
---@param v Kinematic
function Entity:set_kinematic(v) end

---@param self Entity
---@param v PhysicsBody
function Entity:set_physics_body(v) end

---@param self Entity
---@param v Collider
function Entity:set_collider(v) end

---@param self Entity
---@param v Grounded
function Entity:set_grounded(v) end

---@param self Entity
---@param v Velocity
function Entity:set_velocity(v) end

---@param self Entity
---@param v Player
function Entity:set_player(v) end

---@param self Entity
---@param v CurrentRoom
function Entity:set_current_room(v) end

---@param self Entity
---@param v Layer
function Entity:set_layer(v) end

---@param self Entity
---@param v Position
function Entity:set_position(v) end

---@param self Entity
---@param v RoomCamera
function Entity:set_room_camera(v) end

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

---@vararg any Arguments passed to the entity's interact function
---@return nil
function Entity:interact(...) end

---@return Entity|nil
function Entity:find_best_interactable() end

return Entity
