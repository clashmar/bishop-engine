use serde::{Deserialize, Serialize};

/// Background panel for menus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuPanel {
}

impl Default for MenuPanel {
    fn default() -> Self {
        Self {}
    }
}
