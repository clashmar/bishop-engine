use serde::{Deserialize, Serialize};

/// Represents the current state of the menu system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MenuMode {
    /// No menu active, game runs normally.
    #[default]
    Running,
    /// Game paused, visible in background.
    Paused,
    /// Full black screen, game hidden.
    BlackScreen,
    /// Overlay menu, game continues.
    Overlay,
}

impl MenuMode {
    /// Returns true if the game logic should be updated.
    pub fn is_game_running(&self) -> bool {
        matches!(self, MenuMode::Running | MenuMode::Overlay)
    }

    /// Returns true if the game should be rendered.
    pub fn is_game_visible(&self) -> bool {
        !matches!(self, MenuMode::BlackScreen)
    }
}
