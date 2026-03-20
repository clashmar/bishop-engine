// game/game_global.rs
use crate::scripting::commands::lua_command_manager::LuaCommandManager;
use crate::scripting::commands::lua_command::LuaCommand;
use crate::input::input_snapshot::InputSnapshot;
use std::cell::{Cell, RefCell};
use std::vec::IntoIter;
use std::rc::Rc;

/// Global services for the `GameState`.
#[derive(Default)]
pub struct GameServices {
    pub command_manager: RefCell<LuaCommandManager>,
    pub input_snapshot: RefCell<InputSnapshot>,
    pub menu_active: Cell<bool>,
}

thread_local! {
    static GAME_SERVICES: Rc<GameServices> = Rc::new(GameServices::default());
}

/// Push an `LuaCommand` to the global command queue.
pub fn push_command(cmd: Box<dyn LuaCommand>) {
    GAME_SERVICES.with(|services| {
        services.command_manager.borrow_mut().push(cmd);
    });
}

/// Consumes the current contents of the global command queue and returns an iterator.
pub fn drain_commands() -> IntoIter<Box<dyn LuaCommand>> {
    GAME_SERVICES.with(|services| {
        return services.command_manager.borrow_mut().drain();
    })
}

/// Returns a fresh copy of the current `InputSnapshot`.
pub fn get_input_snapshot() -> InputSnapshot {
    GAME_SERVICES.with(|services| services.input_snapshot.borrow().clone())
}

/// Sets whether a menu is currently active.
pub fn set_menu_active(active: bool) {
    GAME_SERVICES.with(|services| {
        services.menu_active.set(active);
    });
}

/// Returns true if any menu is currently active.
pub fn is_menu_active() -> bool {
    GAME_SERVICES.with(|services| services.menu_active.get())
}

