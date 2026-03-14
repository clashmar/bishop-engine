use serde::{Deserialize, Serialize};
use super::menu_element::MenuElement;
use crate::menu::layout::LayoutConfig;

/// Element that arranges its children using layout rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutGroupElement {
    pub layout: LayoutConfig,
    pub children: Vec<LayoutChild>,
}

impl Default for LayoutGroupElement {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),
            children: Vec::new(),
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
