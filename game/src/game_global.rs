// game/game_global.rs
use crate::scripting::commands::lua_command_manager::LuaCommandManager;
use crate::scripting::commands::lua_command::LuaCommand;
use crate::game_state::*;
use engine_core::input::input_snapshot::InputSnapshot;
use std::future::Future;
use std::vec::IntoIter;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

/// Global services for the `GameState`.
pub struct GameServices {
    pub game_state: RefCell<Option<GameState>>, // set once at startup
    pub command_manager: RefCell<LuaCommandManager>,
    pub input_snapshot: RefCell<InputSnapshot>,
}

impl GameServices {
    pub fn new() -> Self {
        Self {
            game_state: RefCell::new(None),
            command_manager: RefCell::new(LuaCommandManager::default()),
            input_snapshot: RefCell::new(InputSnapshot::default()),
        }
    }
}

thread_local! {
    static GAME_SERVICES: Rc<GameServices> = Rc::new(GameServices::new());
}

/// Store the `GameState` in global services.
pub fn set_game(game: GameState) {
    GAME_SERVICES.with(|services| {
        *services.game_state.borrow_mut() = Some(game);
    });
}

/// Gets mutable access to the `GameState`.
pub fn with_game_state<F, R>(f: F) -> R
where
    F: FnOnce(&GameState) -> R,
{
    GAME_SERVICES.with(|services| {
        let opt = services.game_state.borrow();
        let game_state = opt
            .as_ref()
            .clone()
            .expect("GameState not initialised");
        f(game_state)
    })
}


/// Gets mutable access to the `GameState`.
pub fn with_game_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut GameState) -> R,
{
    GAME_SERVICES.with(|services| {
        let mut opt = services.game_state.borrow_mut();
        let game_state = opt
            .as_mut()
            .expect("GameState not initialised");
        f(game_state)
    })
}

/// Gets async mutable access to the `GameState`.
pub async fn with_game_state_mut_async<R, F>(f: F) -> R
where
    for<'a> F: FnOnce(&'a mut GameState) -> Pin<Box<dyn Future<Output = R> + 'a>>,
{
    let services = GAME_SERVICES.with(|s| s.clone());

    let mut opt = services.game_state.borrow_mut();
    let game_state = opt
        .as_mut()
        .expect("GameState not initialised");

    let fut = f(game_state);
    fut.await
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

