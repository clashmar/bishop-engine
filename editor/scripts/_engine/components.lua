-- Auto-generated. Do not edit.
---@meta
---@alias vec2 { x: number, y: number }
---@alias vec3 { x: number, y: number, z: number }

---@class CurrentFrame
---@field clip_id number
---@field col number
---@field row number
---@field offset vec2
---@field sprite_id number
---@field frame_size vec2

---@class Damage
---@field amount number

---@class Solid
---@field value boolean

---@class Walkable
---@field value boolean

---@class Kinematic
--- Marker component

---@class PhysicsBody
--- Marker component

---@class Collider
---@field width number
---@field height number

---@class Grounded
---@field value boolean

---@class Velocity
---@field x number
---@field y number

---@class Player
--- Marker component

---@class CurrentRoom
---@field value number

---@class Layer
---@field z number

---@class Position
---@field position vec2

---@class Global
--- Marker component

---@class Light
---@field pos vec2
---@field color vec3
---@field intensity number
---@field radius number
---@field spread number
---@field alpha number
---@field brightness number

---@class Sprite
---@field sprite number

---@class Script
---@field script_id number
---@field data table

---@class Glow
---@field color vec3
---@field intensity number
---@field brightness number
---@field emission number
---@field sprite_id number

---@class Interactable
---@field range number

---@class Animation
---@field clips table
---@field variant table
---@field current table
---@field states table
---@field sprite_cache table

---@class RoomCamera
---@field zoom vec2
---@field room_id number
---@field zoom_mode table
---@field camera_mode table

---@class ComponentId
---@field CurrentFrame string
---@field Damage string
---@field Solid string
---@field Walkable string
---@field Kinematic string
---@field PhysicsBody string
---@field Collider string
---@field Grounded string
---@field Velocity string
---@field Player string
---@field CurrentRoom string
---@field Layer string
---@field Position string
---@field Global string
---@field Light string
---@field Sprite string
---@field Script string
---@field Glow string
---@field Interactable string
---@field Animation string
---@field RoomCamera string

local C = {}

C.CurrentFrame = "CurrentFrame"
C.Damage = "Damage"
C.Solid = "Solid"
C.Walkable = "Walkable"
C.Kinematic = "Kinematic"
C.PhysicsBody = "PhysicsBody"
C.Collider = "Collider"
C.Grounded = "Grounded"
C.Velocity = "Velocity"
C.Player = "Player"
C.CurrentRoom = "CurrentRoom"
C.Layer = "Layer"
C.Position = "Position"
C.Global = "Global"
C.Light = "Light"
C.Sprite = "Sprite"
C.Script = "Script"
C.Glow = "Glow"
C.Interactable = "Interactable"
C.Animation = "Animation"
C.RoomCamera = "RoomCamera"

return C
