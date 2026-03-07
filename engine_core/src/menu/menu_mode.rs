use serde::{Deserialize, Serialize};

/// Represents the menu mode for a given menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MenuMode {
    #[default]
    /// Game paused, visible in background.
    Paused,
    /// Full black screen, game hidden.
    BlackScreen,
    /// Overlay menu, game continues.
    Overlay,
    /// Menu takes up full dimensions of screen.
    FullScreen,
}

impl MenuMode {
    /// Returns true if the game logic should be pause.
    pub fn is_paused(&self) -> bool {
        matches!(self, MenuMode::Paused | MenuMode::BlackScreen | MenuMode::FullScreen)
    }

    /// Returns true if the game is hidden by a menu.
    pub fn is_hiding_game(&self) -> bool {
        !matches!(self, MenuMode::BlackScreen | MenuMode::FullScreen)
    }
}
