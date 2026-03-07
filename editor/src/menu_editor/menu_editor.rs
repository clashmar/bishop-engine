// editor/src/menu_editor/menu_editor.rs
use bishop::prelude::*;
use engine_core::prelude::*;

/// Main menu editor state.
pub struct MenuEditor {
    pub templates: Vec<MenuTemplate>,
    pub current_template_index: Option<usize>,
    pub selected_element_index: Option<usize>,
    pub pending_element_type: Option<MenuElementKind>,
}

impl MenuEditor {
    /// Creates a new menu editor.
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            current_template_index: None,
            selected_element_index: None,
            pending_element_type: None,
        }
    }

    /// Returns a reference to the current template.
    pub fn current_template(&self) -> Option<&MenuTemplate> {
        self.current_template_index
            .and_then(|i| self.templates.get(i))
    }

    /// Returns a mutable reference to the current template.
    pub fn current_template_mut(&mut self) -> Option<&mut MenuTemplate> {
        self.current_template_index
            .and_then(|i| self.templates.get_mut(i))
    }

    /// Sets all templates and selects the first one if available.
    pub fn set_templates(&mut self, templates: Vec<MenuTemplate>) {
        self.templates = templates;
        self.current_template_index = if self.templates.is_empty() {
            None
        } else {
            Some(0)
        };
        self.selected_element_index = None;
    }

    /// Selects a template by index.
    pub fn select_template(&mut self, index: usize) {
        if index < self.templates.len() {
            self.current_template_index = Some(index);
            self.selected_element_index = None;
        }
    }

    /// Creates a new menu template with the given id.
    pub fn create_new_template(&mut self, id: String) {
        let template = MenuTemplate::new(id);
        self.templates.push(template);
        self.current_template_index = Some(self.templates.len() - 1);
        self.selected_element_index = None;
    }

    /// Deletes the template at the given index.
    pub fn delete_template(&mut self, index: usize) {
        if index >= self.templates.len() {
            return;
        }
        self.templates.remove(index);

        if self.templates.is_empty() {
            self.current_template_index = None;
        } else if let Some(current) = self.current_template_index {
            if current >= self.templates.len() {
                self.current_template_index = Some(self.templates.len() - 1);
            } else if current > index {
                self.current_template_index = Some(current - 1);
            }
        }
        self.selected_element_index = None;
    }

    /// Adds an element to the current template at the given position.
    pub fn add_element(&mut self, kind: MenuElementKind, position: Vec2) {
        let Some(template) = self.current_template_mut() else {
            return;
        };

        let default_size = match &kind {
            MenuElementKind::Label(_) => Vec2::new(200.0, 32.0),
            MenuElementKind::Button(_) => Vec2::new(200.0, 40.0),
            MenuElementKind::Spacer(s) => Vec2::new(200.0, s.size),
            MenuElementKind::Panel(_) => Vec2::new(300.0, 200.0),
        };

        let rect = Rect::new(position.x, position.y, default_size.x, default_size.y);
        let element = MenuElement::new(kind, rect);
        template.elements.push(element);
        self.selected_element_index = Some(template.elements.len() - 1);
    }

    /// Deletes the currently selected element.
    pub fn delete_selected_element(&mut self) {
        let Some(index) = self.selected_element_index else {
            return;
        };
        let Some(template) = self.current_template_mut() else {
            return;
        };

        if index >= template.elements.len() {
            return;
        }

        template.elements.remove(index);

        if template.elements.is_empty() {
            self.selected_element_index = None;
        } else if index >= template.elements.len() {
            self.selected_element_index = Some(template.elements.len() - 1);
        }
    }

    /// Returns a reference to the selected element.
    pub fn selected_element(&self) -> Option<&MenuElement> {
        let template = self.current_template()?;
        let index = self.selected_element_index?;
        template.elements.get(index)
    }

    /// Returns a mutable reference to the selected element.
    pub fn selected_element_mut(&mut self) -> Option<&mut MenuElement> {
        let index = self.selected_element_index?;
        self.current_template_mut()?.elements.get_mut(index)
    }
}

impl Default for MenuEditor {
    fn default() -> Self {
        Self::new()
    }
}
