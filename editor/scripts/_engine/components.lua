-- Auto-generated. Do not edit.
---@meta
---@alias vec2 { x: number, y: number }
---@alias vec3 { x: number, y: number, z: number }

---@class Script
---@field script_id number
---@field data table

---@class Sprite
---@field sprite number

---@class CurrentFrame
---@field clip_id number
---@field col number
---@field row number
---@field offset vec2
---@field sprite_id number
---@field frame_size vec2

---@class Children
---@field entities table

---@class Parent
---@field value table

---@class Position
---@field position vec2

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

---@class Global
--- Marker component

---@class Name
---@field value string

---@class RoomCamera
---@field zoom vec2
---@field room_id number
---@field zoom_mode table
---@field camera_mode table

---@class Glow
---@field color vec3
---@field intensity number
---@field brightness number
---@field emission number
---@field sprite_id number

---@class Animation
---@field clips table
---@field variant table
---@field current table
---@field states table
---@field sprite_cache table

---@class Interactable
---@field range number

---@class Light
---@field pos vec2
---@field color vec3
---@field intensity number
---@field radius number
---@field spread number
---@field alpha number
---@field brightness number

---@class ComponentId
---@field Script string
---@field Sprite string
---@field CurrentFrame string
---@field Children string
---@field Parent string
---@field Position string
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
---@field Global string
---@field Name string
---@field RoomCamera string
---@field Glow string
---@field Animation string
---@field Interactable string
---@field Light string

local C = {}

C.Script = "Script"
C.Sprite = "Sprite"
C.CurrentFrame = "CurrentFrame"
C.Children = "Children"
C.Parent = "Parent"
C.Position = "Position"
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
C.Global = "Global"
C.Name = "Name"
C.RoomCamera = "RoomCamera"
C.Glow = "Glow"
C.Animation = "Animation"
C.Interactable = "Interactable"
C.Light = "Light"

return C
