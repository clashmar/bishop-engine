use serde::{Deserialize, Serialize};
use crate::menu::menu_builder::MenuAction;

/// Clickable button component.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuButton {
    pub text: String,
    pub action: MenuAction,
}

impl Default for MenuAction {
    fn default() -> Self {
        MenuAction::Custom(String::new())
    }
}
