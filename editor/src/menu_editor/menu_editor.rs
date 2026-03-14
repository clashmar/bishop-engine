// editor/src/menu_editor/menu_editor.rs
use crate::gui::panels::panel_manager::is_mouse_over_panel;
use crate::gui::modal::is_modal_open;
use crate::menu_editor::*;
use crate::menu_editor::resize_handle::ResizeHandleState;
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
    pub selected_child_index: Option<usize>,
    pub pending_element_type: Option<MenuElementKind>,
    pub(crate) active_rects: Vec<Rect>,
    pub(crate) dragging_element: Option<usize>,
    pub(crate) drag_offset: Vec2,
    pub(crate) resizing_handle: Option<ResizeHandleState>,
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
            selected_child_index: None,
            pending_element_type: None,
            active_rects: Vec::new(),
            dragging_element: None,
            drag_offset: Vec2::ZERO,
            resizing_handle: None,
        }
    }

    /// Updates the menu editor and handles input.
    pub fn update(
        &mut self,
        ctx: &mut WgpuContext,
        camera: &Camera2D,
    ) {
        let canvas_rect = compute_canvas_rect(ctx.screen_width(), ctx.screen_height());

        let blocked = self.is_mouse_over_ui(ctx);

        self.update_canvas(ctx, camera, canvas_rect, blocked);
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
        self.draw_canvas(ctx, camera, canvas_rect);

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
        self.selected_child_index = None;
    }

    /// Selects a template by index.
    pub fn select_template(&mut self, index: usize) {
        if index < self.templates.len() {
            self.current_template_index = Some(index);
            self.selected_element_index = None;
            self.selected_child_index = None;
        }
    }

    /// Creates a new menu template with the given id.
    pub fn create_new_template(&mut self, id: String) {
        let template = MenuTemplate::new(id);
        self.templates.push(template);
        self.current_template_index = Some(self.templates.len() - 1);
        self.selected_element_index = None;
        self.selected_child_index = None;
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
        self.selected_child_index = None;
    }

    /// Adds an element to the current template at the given position.
    /// If a layout group is selected, adds as a managed child of that group instead.
    pub fn add_element(&mut self, kind: MenuElementKind, position: Vec2) {
        let template_idx = match self.current_template_index {
            Some(i) if i < self.templates.len() => i,
            _ => return,
        };

        let default_size = match &kind {
            MenuElementKind::Label(_) => Vec2::new(0.10, 0.03),
            MenuElementKind::Button(_) => Vec2::new(0.10, 0.037),
            MenuElementKind::Panel(_) => Vec2::new(0.16, 0.185),
            MenuElementKind::LayoutGroup(_) => Vec2::new(0.25, 0.30),
        };

        let template = &mut self.templates[template_idx];

        if let Some(selected_idx) = self.selected_element_index {
            if let Some(selected) = template.elements.get_mut(selected_idx) {
                if let MenuElementKind::LayoutGroup(group) = &mut selected.kind {
                    // Store position relative to the group origin so unmanaged children
                    // render at the expected location when toggled from managed.
                    let group_origin = Vec2::new(selected.rect.x, selected.rect.y);
                    let rel_pos = position - group_origin;
                    let rect = Rect::new(rel_pos.x, rel_pos.y, default_size.x, default_size.y);
                    let element = MenuElement::new(kind, rect);
                    group.children.push(LayoutChild { element, managed: true });
                    return;
                }
            }
        }

        let rect = Rect::new(position.x, position.y, default_size.x, default_size.y);
        let element = MenuElement::new(kind, rect);

        template.elements.push(element);
        self.selected_element_index = Some(template.elements.len() - 1);
    }

    /// Deletes the currently selected element or child.
    pub fn delete_selected_element(&mut self) {
        let Some(index) = self.selected_element_index else {
            return;
        };

        if let Some(child_idx) = self.selected_child_index {
            let template_idx = match self.current_template_index {
                Some(i) if i < self.templates.len() => i,
                _ => return,
            };
            let new_child_idx = {
                let template = &mut self.templates[template_idx];
                if let Some(element) = template.elements.get_mut(index) {
                    if let MenuElementKind::LayoutGroup(group) = &mut element.kind {
                        if child_idx < group.children.len() {
                            group.children.remove(child_idx);
                            if group.children.is_empty() {
                                None
                            } else if child_idx >= group.children.len() {
                                Some(group.children.len() - 1)
                            } else {
                                Some(child_idx)
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            self.selected_child_index = new_child_idx;
            return;
        }

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

    /// Returns a reference to the selected element or child element when a child is selected.
    pub fn selected_element(&self) -> Option<&MenuElement> {
        let template = self.current_template()?;
        let index = self.selected_element_index?;
        let element = template.elements.get(index)?;
        if let Some(child_idx) = self.selected_child_index {
            if let MenuElementKind::LayoutGroup(group) = &element.kind {
                return group.children.get(child_idx).map(|c| &c.element);
            }
        }
        Some(element)
    }

    /// Returns a mutable reference to the selected element or child element when a child is selected.
    pub fn selected_element_mut(&mut self) -> Option<&mut MenuElement> {
        let index = self.selected_element_index?;
        let template_idx = self.current_template_index?;

        if let Some(ci) = self.selected_child_index {
            // Always return here — the `if let` branch never falls through,
            // so the borrow from `self.templates` below is a separate code path.
            return self.templates.get_mut(template_idx)
                .and_then(|t| t.elements.get_mut(index))
                .and_then(|e| {
                    if let MenuElementKind::LayoutGroup(g) = &mut e.kind {
                        g.children.get_mut(ci).map(|c| &mut c.element)
                    } else {
                        None
                    }
                });
        }

        self.templates.get_mut(template_idx)?.elements.get_mut(index)
    }

    /// Returns true when a managed child element is currently selected.
    pub fn is_selected_child_managed(&self) -> bool {
        let Some(child_idx) = self.selected_child_index else { return false };
        let Some(parent_idx) = self.selected_element_index else { return false };
        let Some(template) = self.current_template() else { return false };
        let Some(element) = template.elements.get(parent_idx) else { return false };
        let MenuElementKind::LayoutGroup(group) = &element.kind else { return false };
        group.children.get(child_idx).map(|c| c.managed).unwrap_or(false)
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
