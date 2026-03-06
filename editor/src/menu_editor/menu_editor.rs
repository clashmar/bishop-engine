// editor/src/menu_editor/menu_editor.rs
use engine_core::prelude::*;

/// Editor mode for visual menu composition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuEditorMode {
    Select,
    AddElement,
    Move,
    Delete,
}

impl Default for MenuEditorMode {
    fn default() -> Self {
        Self::Select
    }
}

/// Main menu editor state.
pub struct MenuEditor {
    pub mode: MenuEditorMode,
    pub current_template: Option<MenuTemplate>,
    pub selected_element_index: Option<usize>,
}

impl MenuEditor {
    /// Creates a new menu editor.
    pub fn new() -> Self {
        Self {
            mode: MenuEditorMode::Select,
            current_template: None,
            selected_element_index: None,
        }
    }

    /// Loads a menu template for editing.
    pub fn load_template(&mut self, template: MenuTemplate) {
        self.current_template = Some(template);
        self.selected_element_index = None;
    }

    /// Creates a new blank menu template.
    pub fn new_template(&mut self, id: String) {
        self.current_template = Some(MenuTemplate::new(id));
        self.selected_element_index = None;
    }

    /// Saves the current template.
    pub fn save_template(&self) -> Option<MenuTemplate> {
        self.current_template.clone()
    }

    /// Returns a reference to the selected element.
    pub fn selected_element(&self) -> Option<&MenuElement> {
        let template = self.current_template.as_ref()?;
        let index = self.selected_element_index?;
        template.elements.get(index)
    }

    /// Returns a mutable reference to the selected element.
    pub fn selected_element_mut(&mut self) -> Option<&mut MenuElement> {
        let index = self.selected_element_index?;
        self.current_template.as_mut()?.elements.get_mut(index)
    }
}

impl Default for MenuEditor {
    fn default() -> Self {
        Self::new()
    }
}
