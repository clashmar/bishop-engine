/// Trait for handling custom menu actions.
///
/// Engine-level actions (Resume, OpenMenu, CloseMenu, QuitGame) are handled
/// automatically by MenuManager. Custom actions are forwarded to implementations
/// of this trait, allowing game-specific logic to respond to menu events.
pub trait MenuActionHandler {
    /// Handles a custom menu action.
    ///
    /// Returns true if the action was handled, false otherwise.
    fn handle_action(&mut self, action: &str) -> bool;
}

/// Default no-op implementation.
pub struct NoOpActionHandler;

impl MenuActionHandler for NoOpActionHandler {
    fn handle_action(&mut self, _action: &str) -> bool {
        false
    }
}
