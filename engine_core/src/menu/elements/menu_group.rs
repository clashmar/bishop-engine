use serde::{Deserialize, Serialize};

/// Groups menu elements together with visibility and behavior control.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuGroup {
    pub id: String,
    pub modal: bool,
    pub blocks_render: bool,
    pub visible: bool,
}
