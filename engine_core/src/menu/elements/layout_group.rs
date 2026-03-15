use serde::{Deserialize, Serialize};
use super::menu_element::MenuElement;
use super::menu_panel::PanelBackground;
use crate::menu::layout::LayoutConfig;

/// Element that arranges its children using layout rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutGroupElement {
    pub layout: LayoutConfig,
    pub children: Vec<LayoutChild>,
    /// Optional panel background rendered behind the children.
    #[serde(default)]
    pub background: Option<PanelBackground>,
    /// Navigation target when leaving upward.
    pub nav_up: Option<usize>,
    /// Navigation target when leaving downward.
    pub nav_down: Option<usize>,
    /// Navigation target when leaving left.
    pub nav_left: Option<usize>,
    /// Navigation target when leaving right.
    pub nav_right: Option<usize>,
}

impl Default for LayoutGroupElement {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),
            children: Vec::new(),
            background: None,
            nav_up: None,
            nav_down: None,
            nav_left: None,
            nav_right: None,
        }
    }
}

/// A child element within a layout group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutChild {
    pub element: MenuElement,
    /// When true, position is computed from layout rules.
    /// When false, rect is relative to group origin but not subject to layout.
    pub managed: bool,
}
