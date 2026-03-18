use bishop::prelude::*;
use serde::{Deserialize, Serialize};
use crate::menu::menu_builder::MenuAction;
use crate::menu::layout::HorizontalAlign;
use super::layout_group::LayoutGroupElement;
use super::menu_panel::PanelBackground;

/// Different kinds of menu elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MenuElementKind {
    Label(LabelElement),
    Button(ButtonElement),
    Panel(PanelElement),
    LayoutGroup(LayoutGroupElement),
}

/// Label element displaying text resolved from a text key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelElement {
    #[serde(alias = "text")]
    pub text_key: String,
    pub font_size: f32,
    pub color: Color,
    #[serde(default)]
    pub alignment: HorizontalAlign,
}

impl Default for LabelElement {
    fn default() -> Self {
        Self {
            text_key: String::new(),
            font_size: 24.0,
            color: Color::WHITE,
            alignment: HorizontalAlign::Center,
        }
    }
}

/// Button element that triggers an action when clicked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonElement {
    #[serde(alias = "text")]
    pub text_key: String,
    pub action: MenuAction,
    pub font_size: f32,
    pub nav_up: Option<usize>,
    pub nav_down: Option<usize>,
    pub nav_left: Option<usize>,
    pub nav_right: Option<usize>,
}

impl Default for ButtonElement {
    fn default() -> Self {
        Self {
            text_key: String::new(),
            action: MenuAction::CloseMenu,
            font_size: 20.0,
            nav_up: None,
            nav_down: None,
            nav_left: None,
            nav_right: None,
        }
    }
}

/// Decorative panel element that renders a background fill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelElement {
    pub background: PanelBackground,
}

impl Default for PanelElement {
    fn default() -> Self {
        Self {
            background: PanelBackground::default(),
        }
    }
}

/// Menu element variants with positional data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuElement {
    pub name: String,
    pub kind: MenuElementKind,
    pub rect: Rect,
    pub enabled: bool,
    pub visible: bool,
    pub z_order: i32,
}

impl MenuElement {
    /// Creates a new menu element.
    pub fn new(kind: MenuElementKind, rect: Rect) -> Self {
        Self {
            name: String::new(),
            kind,
            rect,
            enabled: true,
            visible: true,
            z_order: 0,
        }
    }

    /// Creates a label element.
    pub fn label(text_key: String, rect: Rect) -> Self {
        Self::new(
            MenuElementKind::Label(LabelElement {
                text_key,
                ..Default::default()
            }),
            rect,
        )
    }

    /// Creates a button element.
    pub fn button(text_key: String, action: MenuAction, rect: Rect) -> Self {
        Self::new(
            MenuElementKind::Button(ButtonElement {
                text_key,
                action,
                ..Default::default()
            }),
            rect,
        )
    }

    /// Creates a panel element.
    pub fn panel(background: PanelBackground, rect: Rect) -> Self {
        Self::new(
            MenuElementKind::Panel(PanelElement { background }),
            rect,
        )
    }

    /// Creates a layout group element.
    pub fn layout_group(group: LayoutGroupElement, rect: Rect) -> Self {
        Self::new(MenuElementKind::LayoutGroup(group), rect)
    }
}
