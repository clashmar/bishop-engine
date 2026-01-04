-- Auto-generated. Do not edit.
---@meta
---@alias vec2 { x: number, y: number }
---@alias vec3 { x: number, y: number, z: number }

---@class Glow
---@field color vec3
---@field intensity number
---@field brightness number
---@field emission number
---@field sprite_id number

---@class RoomCamera
---@field zoom vec2
---@field room_id number
---@field zoom_mode table
---@field camera_mode table

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

---@class Name
---@field value string

---@class Script
---@field script_id number
---@field data table

---@class Light
---@field pos vec2
---@field color vec3
---@field intensity number
---@field radius number
---@field spread number
---@field alpha number
---@field brightness number

---@class Animation
---@field clips table
---@field variant table
---@field current table
---@field states table
---@field sprite_cache table

---@class Interactable
---@field range number

---@class CurrentFrame
---@field clip_id number
---@field col number
---@field row number
---@field offset vec2
---@field sprite_id number
---@field frame_size vec2

---@class Sprite
---@field sprite number

---@class ComponentId
---@field Glow string
---@field RoomCamera string
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
---@field Name string
---@field Script string
---@field Light string
---@field Animation string
---@field Interactable string
---@field CurrentFrame string
---@field Sprite string

local C = {}

C.Glow = "Glow"
C.RoomCamera = "RoomCamera"
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
C.Name = "Name"
C.Script = "Script"
C.Light = "Light"
C.Animation = "Animation"
C.Interactable = "Interactable"
C.CurrentFrame = "CurrentFrame"
C.Sprite = "Sprite"

return C
