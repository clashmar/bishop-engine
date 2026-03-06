use engine_core::menu::MenuActionHandler;
use std::cell::RefCell;

thread_local! {
    static MENU_EVENTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// Handles custom menu actions by queuing events to be emitted to Lua.
pub struct GameMenuHandler;

impl MenuActionHandler for GameMenuHandler {
    fn handle_action(&mut self, action: &str) -> bool {
        MENU_EVENTS.with(|events| {
            events.borrow_mut().push(action.to_string());
        });
        true
    }
}

/// Drains all pending menu events and returns them.
pub fn drain_menu_events() -> Vec<String> {
    MENU_EVENTS.with(|events| {
        events.borrow_mut().drain(..).collect()
    })
}
