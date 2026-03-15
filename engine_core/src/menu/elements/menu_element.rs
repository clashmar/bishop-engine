use bishop::prelude::*;
use serde::{Deserialize, Serialize};
use crate::menu::menu_builder::MenuAction;
use super::layout_group::LayoutGroupElement;

/// Different kinds of menu elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MenuElementKind {
    Label(LabelElement),
    Button(ButtonElement),
    Panel(PanelElement),
    LayoutGroup(LayoutGroupElement),
}

/// Label element displaying static text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelElement {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

impl Default for LabelElement {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 24.0,
            color: Color::WHITE,
        }
    }
}

/// Button element that triggers an action when clicked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonElement {
    pub text: String,
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
            text: String::new(),
            action: MenuAction::CloseMenu,
            font_size: 20.0,
            nav_up: None,
            nav_down: None,
            nav_left: None,
            nav_right: None,
        }
    }
}

/// Panel element containing nested elements with its own layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelElement {
    pub elements: Vec<MenuElement>,
    pub background_color: Option<Color>,
}

impl Default for PanelElement {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            background_color: None,
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
    pub fn label(text: String, rect: Rect) -> Self {
        Self::new(
            MenuElementKind::Label(LabelElement {
                text,
                ..Default::default()
            }),
            rect,
        )
    }

    /// Creates a button element.
    pub fn button(text: String, action: MenuAction, rect: Rect) -> Self {
        Self::new(
            MenuElementKind::Button(ButtonElement {
                text,
                action,
                ..Default::default()
            }),
            rect,
        )
    }

    /// Creates a panel element.
    pub fn panel(elements: Vec<MenuElement>, rect: Rect) -> Self {
        Self::new(
            MenuElementKind::Panel(PanelElement {
                elements,
                background_color: None,
            }),
            rect,
        )
    }

    /// Creates a layout group element.
    pub fn layout_group(group: LayoutGroupElement, rect: Rect) -> Self {
        Self::new(MenuElementKind::LayoutGroup(group), rect)
    }
}
