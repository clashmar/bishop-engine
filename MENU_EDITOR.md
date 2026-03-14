# Overview
- Continue to improve menu/ui editor
- A menu can be a basic pause/settings/dialogue branching etc
- Canvas represents the screen
- Composable menus for the game that reuse existing widgets (dogfooding with the editor)
- Changes to widgets are allowed but ask first
- Some of the menu code is unused/unsuitable. Ask questions if you aren't sure what is essential
- Keep decoupled from ECS
- Menu behaviour ought to be configuarable in lua
- Menu manager builds default menus as placeholders for now

# Relevant files/folders
- engine_core/src/menu
- editor/src/menu_editor
- game/src/engine.rs
- widgets/src/widgets

# Long Term Goals
- Menus will have relationships e.g. start menu => settings and back
- Elements should be navigable with a gamepad 
- Widgets will eventually have more customization (e.g. button styling)
- Elements such as panels or the background could have a plain color/sprite/animation/shaders
