# Menu System Integration with Lua

## Overview

PR 2 integrates the menu system with Lua scripts through an event-based system. Custom menu actions are emitted as events that Lua scripts can listen to.

## How It Works

1. When a custom menu action is triggered, it's queued in the game's menu handler
2. During the engine update loop, queued events are drained and emitted to the Lua event bus
3. Lua scripts can register handlers for these events using `engine.on()`

## Lua Integration Example

### Listening to Menu Events

```lua
-- In your Lua script (e.g., scripts/main.lua)

-- Listen for a custom menu action
engine.on("menu:new_game", function()
    print("New game started!")
    -- Initialize new game state
end)

engine.on("menu:options", function()
    print("Options menu opened!")
    -- Show options screen
end)

engine.on("menu:quit", function()
    print("Quit selected!")
    -- Handle quit logic
end)
```

### Registering Custom Menus

To register a menu with custom actions in the engine:

```rust
// In engine initialization or game setup
let custom_menu = MenuBuilder::new("main_menu")
    .screen_size(800.0, 600.0)
    .background(MenuBackground::SolidColor(Color::BLACK))
    .vertical()
    .label("MAIN MENU")
    .spacer(16.0)
    .button("New Game", MenuAction::Custom("new_game".to_string()))
    .button("Options", MenuAction::Custom("options".to_string()))
    .button("Quit", MenuAction::Custom("quit".to_string()))
    .build();

menu_manager.register_template(custom_menu);
```

## Default Menus

The following menus are registered by default:

- **pause**: Simple pause menu with a "Resume" button (opens with P key or Escape)

## Built-in Menu Actions

The following actions are handled automatically by the MenuManager:

- `MenuAction::Resume` - Closes all menus and resumes the game
- `MenuAction::OpenMenu(id)` - Opens a menu by its id
- `MenuAction::CloseMenu` - Closes the current menu
- `MenuAction::QuitToMainMenu` - Closes all menus (game-specific logic in Lua)
- `MenuAction::QuitGame` - Closes all menus (game-specific logic in Lua)
- `MenuAction::Custom(action)` - Emits `menu:{action}` event to Lua

## Event Format

All custom menu actions are emitted with the prefix `menu:`. For example:

- Custom action `"new_game"` → Lua event `"menu:new_game"`
- Custom action `"save_game"` → Lua event `"menu:save_game"`
- Custom action `"load_slot_1"` → Lua event `"menu:load_slot_1"`

## Testing

To test the menu integration:

1. Run the game with `cargo run -p game --features wgpu`
2. Press P or Escape to open the pause menu
3. Use arrow keys to navigate
4. Press Enter to select an option
5. Check that menu events are logged in Lua scripts
