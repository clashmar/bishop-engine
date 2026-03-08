// editor/src/menu_editor/menu_editor.rs
use crate::gui::panels::panel_manager::is_mouse_over_panel;
use crate::gui::modal::is_modal_open;
use crate::menu_editor::*;
use crate::storage::editor_storage::delete_menu;
use engine_core::prelude::*;
use bishop::prelude::*;

/// Main menu editor state.
pub struct MenuEditor {
    pub(crate) menu_list_panel: MenuListPanel,
    pub(crate) element_palette: ElementPalette,
    pub(crate) properties_panel: MenuPropertiesPanel,
    pub templates: Vec<MenuTemplate>,
    pub current_template_index: Option<usize>,
    pub selected_element_index: Option<usize>,
    pub pending_element_type: Option<MenuElementKind>,
    pub(crate) active_rects: Vec<Rect>,
    pub(crate) dragging_element: Option<usize>,
    pub(crate) drag_offset: Vec2,
}

impl MenuEditor {
    /// Creates a new menu editor.
    pub fn new() -> Self {
        Self {
            menu_list_panel: MenuListPanel::new(),
            element_palette: ElementPalette::new(),
            properties_panel: MenuPropertiesPanel::new(),
            templates: Vec::new(),
            current_template_index: None,
            selected_element_index: None,
            pending_element_type: None,
            active_rects: Vec::new(),
            dragging_element: None,
            drag_offset: Vec2::ZERO,
        }
    }

    /// Updates the menu editor and handles input.
    pub fn update(
        &mut self,
        ctx: &mut WgpuContext,
    ) {
        let canvas_rect = compute_canvas_rect(ctx.screen_width(), ctx.screen_height());

        let blocked = self.is_mouse_over_ui(ctx);

        self.update_canvas(ctx, canvas_rect, blocked);
    }

    pub fn draw(
        &mut self,
        ctx: &mut WgpuContext,
        camera: &Camera2D,
        game: &mut Game,
    ) {
        ctx.set_camera(camera);
        ctx.clear_background(Color::BLACK);

        let canvas_rect = compute_canvas_rect(ctx.screen_width(), ctx.screen_height());

        // Draw canvas under ui
        self.draw_canvas(ctx, canvas_rect);

        // Draw ui after canvas
        self.draw_ui(ctx);
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

        if let Err(err) = delete_menu(&self.templates[index].id) {
            onscreen_error!("Error deleting template: {err}");
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
            MenuElementKind::Label(_) => Vec2::new(0.10, 0.03),
            MenuElementKind::Button(_) => Vec2::new(0.10, 0.037),
            MenuElementKind::Panel(_) => Vec2::new(0.16, 0.185),
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

    #[inline]
    pub fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }

    /// Initializes the camera centered on the canvas with a 1:1 screen-space mapping.
    pub fn init_camera(ctx: &WgpuContext, camera: &mut Camera2D) {
        let sw = ctx.screen_width();
        let sh = ctx.screen_height();
        camera.target = Vec2::new(sw / 2.0, sh / 2.0);
        camera.zoom = Vec2::new(2.0 / sw, 2.0 / sh);
        camera.rotation = 0.0;
        camera.offset = Vec2::ZERO;
    }

    pub fn is_mouse_over_ui(&self, ctx: &WgpuContext,) -> bool {
        let mouse_screen: Vec2 = ctx.mouse_position().into();
        self.active_rects.iter().any(|r| r.contains(mouse_screen))
            || is_dropdown_open()
            || is_modal_open()
            || is_mouse_over_panel(ctx)
    }
}

impl Default for MenuEditor {
    fn default() -> Self {
        Self::new()
    }
}
