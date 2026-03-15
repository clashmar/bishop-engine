-- Auto-generated. Do not edit.
---@meta
---@alias vec2 { x: number, y: number }
---@alias vec3 { x: number, y: number, z: number }

---@alias Name string

---@class PhysicsBody
--- Marker component

---@class PlayerProxy
--- Marker component

---@class Kinematic
--- Marker component

---@class Global
--- Marker component

---@class Layer
---@field z number

---@alias CurrentRoom number

---@class Player
--- Marker component

---@class SubPixel
---@field x number
---@field y number

---@alias Solid boolean

---@class Damage
---@field amount number

---@class Velocity
---@field x number
---@field y number

---@alias Walkable boolean

---@alias Grounded boolean

---@class Collider
---@field width number
---@field height number

---@alias FacingDirection table

---@class CurrentFrame
---@field clip_id number
---@field col number
---@field row number
---@field offset vec2
---@field sprite_id number
---@field frame_size vec2
---@field flip_x boolean

---@class SpeechBubble
---@field text string
---@field timer number
---@field color table
---@field offset table
---@field font_size table
---@field max_width table
---@field show_background boolean
---@field background_color table

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

---@class Sprite
---@field sprite number

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

---@class Interactable
---@field range number

---@class Transform
---@field visible boolean
---@field position vec2
---@field pivot table

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

---@class ComponentId
---@field Name string
---@field PhysicsBody string
---@field PlayerProxy string
---@field Kinematic string
---@field Global string
---@field Layer string
---@field CurrentRoom string
---@field Player string
---@field SubPixel string
---@field Solid string
---@field Damage string
---@field Velocity string
---@field Walkable string
---@field Grounded string
---@field Collider string
---@field FacingDirection string
---@field CurrentFrame string
---@field SpeechBubble string
---@field RoomCamera string
---@field Glow string
---@field Sprite string
---@field Children string
---@field Parent string
---@field Animation string
---@field Interactable string
---@field Transform string
---@field Script string
---@field Light string

local C = {}

C.Name = "Name"
C.PhysicsBody = "PhysicsBody"
C.PlayerProxy = "PlayerProxy"
C.Kinematic = "Kinematic"
C.Global = "Global"
C.Layer = "Layer"
C.CurrentRoom = "CurrentRoom"
C.Player = "Player"
C.SubPixel = "SubPixel"
C.Solid = "Solid"
C.Damage = "Damage"
C.Velocity = "Velocity"
C.Walkable = "Walkable"
C.Grounded = "Grounded"
C.Collider = "Collider"
C.FacingDirection = "FacingDirection"
C.CurrentFrame = "CurrentFrame"
C.SpeechBubble = "SpeechBubble"
C.RoomCamera = "RoomCamera"
C.Glow = "Glow"
C.Sprite = "Sprite"
C.Children = "Children"
C.Parent = "Parent"
C.Animation = "Animation"
C.Interactable = "Interactable"
C.Transform = "Transform"
C.Script = "Script"
C.Light = "Light"

return C
