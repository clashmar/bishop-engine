-- Auto-generated. Do not edit.
---@meta
---@alias vec2 { x: number, y: number }
---@alias vec3 { x: number, y: number, z: number }

---@class Interactable
---@field range number

---@class Glow
---@field color vec3
---@field intensity number
---@field brightness number
---@field emission number
---@field sprite_id number

---@class Light
---@field pos vec2
---@field color vec3
---@field intensity number
---@field radius number
---@field spread number
---@field alpha number
---@field brightness number

---@class Transform
---@field position vec2
---@field pivot table

---@class CurrentFrame
---@field clip_id number
---@field col number
---@field row number
---@field offset vec2
---@field sprite_id number
---@field frame_size vec2
---@field flip_x boolean

---@class Children
---@field entities table

---@alias Parent table

---@class Animation
---@field clips table
---@field variant table
---@field current table
---@field states table
---@field sprite_cache table
---@field flip_x boolean
---@field speed_multiplier number

---@class RoomCamera
---@field zoom vec2
---@field room_id number
---@field zoom_mode table
---@field camera_mode table

---@alias Grounded boolean

---@class Player
--- Marker component

---@class Damage
---@field amount number

---@alias Solid boolean

---@class Collider
---@field width number
---@field height number

---@class Kinematic
--- Marker component

---@class PhysicsBody
--- Marker component

---@class Layer
---@field z number

---@class Velocity
---@field x number
---@field y number

---@alias Name string

---@class Global
--- Marker component

---@alias Walkable boolean

---@alias CurrentRoom number

---@alias FacingDirection table

---@class Script
---@field script_id number
---@field data table

---@class Sprite
---@field sprite number

---@class ComponentId
---@field Interactable string
---@field Glow string
---@field Light string
---@field Transform string
---@field CurrentFrame string
---@field Children string
---@field Parent string
---@field Animation string
---@field RoomCamera string
---@field Grounded string
---@field Player string
---@field Damage string
---@field Solid string
---@field Collider string
---@field Kinematic string
---@field PhysicsBody string
---@field Layer string
---@field Velocity string
---@field Name string
---@field Global string
---@field Walkable string
---@field CurrentRoom string
---@field FacingDirection string
---@field Script string
---@field Sprite string

local C = {}

C.Interactable = "Interactable"
C.Glow = "Glow"
C.Light = "Light"
C.Transform = "Transform"
C.CurrentFrame = "CurrentFrame"
C.Children = "Children"
C.Parent = "Parent"
C.Animation = "Animation"
C.RoomCamera = "RoomCamera"
C.Grounded = "Grounded"
C.Player = "Player"
C.Damage = "Damage"
C.Solid = "Solid"
C.Collider = "Collider"
C.Kinematic = "Kinematic"
C.PhysicsBody = "PhysicsBody"
C.Layer = "Layer"
C.Velocity = "Velocity"
C.Name = "Name"
C.Global = "Global"
C.Walkable = "Walkable"
C.CurrentRoom = "CurrentRoom"
C.FacingDirection = "FacingDirection"
C.Script = "Script"
C.Sprite = "Sprite"

return C
