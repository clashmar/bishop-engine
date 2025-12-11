// game/game_global.rs
use crate::scripting::commands::lua_command_manager::LuaCommandManager;
use crate::scripting::commands::lua_command::LuaCommand;
use engine_core::input::input_snapshot::InputSnapshot;
use std::vec::IntoIter;
use std::cell::RefCell;
use std::rc::Rc;

/// Global services for the `GameState`.
pub struct GameServices {
    pub command_manager: RefCell<LuaCommandManager>,
    pub input_snapshot: RefCell<InputSnapshot>,
}

impl GameServices {
    pub fn new() -> Self {
        Self {
            command_manager: RefCell::new(LuaCommandManager::default()),
            input_snapshot: RefCell::new(InputSnapshot::default()),
        }
    }
}

thread_local! {
    static GAME_SERVICES: Rc<GameServices> = Rc::new(GameServices::new());
}

/// Push an `LuaCommand` to the global command queue.
pub fn push_command(cmd: Box<dyn LuaCommand>) {
    GAME_SERVICES.with(|services| {
        println!("Pushed");
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

