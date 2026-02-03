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
---@field flip_x boolean

---@class FacingDirection
---@field value table

---@class Interactable
---@field range number

---@class Sprite
---@field sprite number

---@class Children
---@field entities table

---@class Parent
---@field value table

---@class Glow
---@field color vec3
---@field intensity number
---@field brightness number
---@field emission number
---@field sprite_id number

---@class Transform
---@field position vec2

---@class RoomCamera
---@field zoom vec2
---@field room_id number
---@field zoom_mode table
---@field camera_mode table

---@class Animation
---@field clips table
---@field variant table
---@field current table
---@field states table
---@field sprite_cache table
---@field flip_x boolean
---@field speed_multiplier number

---@class Grounded
---@field value boolean

---@class Player
--- Marker component

---@class Damage
---@field amount number

---@class Solid
---@field value boolean

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

---@class Name
---@field value string

---@class Global
--- Marker component

---@class Walkable
---@field value boolean

---@class CurrentRoom
---@field value number

---@class Light
---@field pos vec2
---@field color vec3
---@field intensity number
---@field radius number
---@field spread number
---@field alpha number
---@field brightness number

---@class Script
---@field script_id number
---@field data table

---@class ComponentId
---@field CurrentFrame string
---@field FacingDirection string
---@field Interactable string
---@field Sprite string
---@field Children string
---@field Parent string
---@field Glow string
---@field Transform string
---@field RoomCamera string
---@field Animation string
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
---@field Light string
---@field Script string

local C = {}

C.CurrentFrame = "CurrentFrame"
C.FacingDirection = "FacingDirection"
C.Interactable = "Interactable"
C.Sprite = "Sprite"
C.Children = "Children"
C.Parent = "Parent"
C.Glow = "Glow"
C.Transform = "Transform"
C.RoomCamera = "RoomCamera"
C.Animation = "Animation"
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
C.Light = "Light"
C.Script = "Script"

return C
