use serde::{Deserialize, Serialize};
use crate::menu::menu_background::MenuBackground;

/// Background panel for menus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuPanel {
    pub background: MenuBackground,
}

impl Default for MenuPanel {
    fn default() -> Self {
        Self {
            background: MenuBackground::default(),
        }
    }
}
