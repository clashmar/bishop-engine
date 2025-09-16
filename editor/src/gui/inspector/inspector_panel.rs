// editor/src/gui/inspector/inspector_panel.rs
use engine_core::ecs::component::Player;
use macroquad::prelude::*;
use engine_core::ui::widgets::*;
use engine_core::{
    assets::asset_manager::AssetManager,
    ecs::{
        component_registry::{COMPONENTS, ComponentReg},
        entity::Entity,
        module::{CollapsibleModule, InspectorModule},
        module_factory::MODULES,
        world_ecs::WorldEcs,
    },
};
use crate::gui::inspector::player_module::PlayerModule;
use crate::gui::inspector::transform_module::TransformModule;

/// The panel that lives on the right‑hand side of the room editor window
pub struct InspectorPanel {
    /// Geometry of the panel
    rect: Rect,
    /// Currently inspected entity
    pub target: Option<Entity>,
    /// All sub‑modules that can draw UI
    modules: Vec<Box<dyn InspectorModule>>,
    /// If true hide normal panel and show only the add‑component UI
    add_mode: bool,
    /// Component name that the user selected from the menu
    pending_add: Option<String>,
    /// Rectangles that were drawn this frame and are therefore active.
    active_rects: Vec<Rect>,
}

impl InspectorPanel {
    /// Create a fresh panel with the default set of modules
    pub fn new() -> Self {
        let mut modules: Vec<Box<dyn InspectorModule>> = Vec::new();

        // Wrap each concrete module in a CollapsibleModule
        modules.push(Box::new(
            PlayerModule::default(),
        ));

        // Wrap each concrete module in a CollapsibleModule
        modules.push(Box::new(
            CollapsibleModule::new(TransformModule::default()).with_title("Transform"),
        ));

        // Add generic modules here
        for entry in MODULES.iter() {
            modules.push((entry.factory)());
        }
        
        Self {
            rect: Rect::new(0., 0., 0., 0.),
            target: None,
            modules,
            add_mode: false,
            pending_add: None,
            active_rects: Vec::new(),
        }
    }

    /// Called by the editor each frame to place the panel
    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    /// Tell the inspector which entity is currently selected
    pub fn set_target(&mut self, entity: Option<Entity>) {
        self.target = entity;
    }

    /// Render the panel and any visible sub‑modules
    /// Returns true if 'Create' was pressed
    pub fn draw(
        &mut self,
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
    ) -> bool {
        self.active_rects.clear();

        const INSET: f32 = 10.0;      
        const BTN_HEIGHT: f32 = 30.0;
        const BTN_MARGIN: f32 = 10.0;
        const SPACING: f32 = 10.0;    
        const PADDING: f32 = 20.0;

        // When an entity is selected we show “Remove” and “Add Component”
        if let Some(entity) = self.target {
            // Labels
            let remove_label = "Remove";
            let add_label = "Add Component";

            // Measure text to obtain proper button widths
            let txt_remove = measure_text(remove_label, None, 20, 1.0);
            let txt_add = measure_text(add_label, None, 20, 1.0);
            let btn_w_remove = txt_remove.width + PADDING;
            let btn_w_add = txt_add.width + PADDING;

            // Compute left‑most X so the pair stays inside the screen
            let total_w = btn_w_remove + btn_w_add + SPACING;
            let x_start = screen_width() - INSET - total_w;

            // Add Component button
            let add_rect = self.register_rect(Rect::new(
                x_start + btn_w_remove + SPACING,
                INSET,
                btn_w_add,
                BTN_HEIGHT,
            ));
            
            if gui_button(add_rect, add_label) {
                if self.can_show_any_component(world_ecs) {
                    self.add_mode = !self.add_mode;
                }
            }

            // Remove button
            // Don't show remove for player entity
            if !world_ecs.get_store::<Player>().contains(entity) {
                let remove_rect = self.register_rect(Rect::new(x_start, INSET, btn_w_remove, BTN_HEIGHT));

                if gui_button(remove_rect, remove_label) {
                    world_ecs.remove_entity(entity);
                    self.target = None;
                    self.add_mode = false;
                    return false;
                }
            }

            // Draw the drop‑down menu when in add mode
            if self.add_mode {
                self.draw_add_component_menu(add_rect, world_ecs);
            }

            // Normal inspector UI (hidden while add_mode is true)
            if !self.add_mode {
                // Compute the top offset for the panel
                let top_offset = add_rect.y + BTN_HEIGHT + INSET;

                // Reduce the height so the panel still fits
                let inner = Rect::new(
                    self.rect.x,
                    top_offset,
                    self.rect.w - INSET,
                    self.rect.h - (top_offset - self.rect.y) - INSET,
                );

                // Background
                draw_rectangle(
                    inner.x,
                    inner.y,
                    inner.w,
                    inner.h,
                    Color::new(0., 0., 0., 0.6),
                );

                // Outline 
                draw_rectangle_lines(inner.x, inner.y, inner.w, inner.h, 2., WHITE);

                // Layout the modules vertically
                let mut y = inner.y + 10.0;
                for module in &mut self.modules {
                    if module.visible(world_ecs, entity) {
                        let h = module.height(); // dynamic height
                        let sub_rect = Rect::new(inner.x + 10.0, y, inner.w - 20.0, h);
                        module.draw(sub_rect, asset_manager, world_ecs, entity);
                        y += h + 10.0;
                    }
                }
            }
        } else {
            // No entity selected – show the old “Create” button
            let create_label = "Create";
            let txt_create = measure_text(create_label, None, 20, 1.0);
            let button = self.register_rect(Rect::new(
                self.rect.x + self.rect.w - txt_create.width - BTN_MARGIN - PADDING,
                self.rect.y + BTN_MARGIN,
                txt_create.width + PADDING,
                BTN_HEIGHT,
            ));
            return gui_button(button, create_label);
        }

        // Process pending component addition
        if let (Some(name), Some(entity)) = (self.pending_add.take(), self.target) {
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == name) {
                (reg.factory)(world_ecs, entity);
            } else {
                eprintln!("Component `{}` not found in registry", name);
            }
        }
        false
    }

    /// Draw the drop‑down list that appears under the Add Component button
    fn draw_add_component_menu(&mut self, button_rect: Rect, world_ecs: &mut WorldEcs) {
        let entity = match self.target {
            Some(e) => e,
            None => return,
        };
        // Collect the components that can be added
        let mut shown: Vec<&ComponentReg> = Vec::new();

        for entry in MODULES.iter() {
            let type_name = entry.title;
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == type_name) {
                if !entity_has_component(world_ecs, entity, reg) {
                    shown.push(reg);
                }
            } else {
                eprintln!("Module `{}` has no ComponentReg entry", type_name);
            }
        }
        // Close the menu if nothing to show
        if shown.is_empty() {
            self.add_mode = false;
            return;
        }
        const ENTRY_H: f32 = 30.0;
        const DEFAULT_MENU_W: f32 = 200.0;
        const MIN_INSET: f32 = 10.0;

        // Determine needed width (widest entry + padding)
        let mut needed_w = DEFAULT_MENU_W;
        for reg in &shown {
            let txt = measure_text(reg.type_name, None, 20, 1.0);
            let w = txt.width + 20.0;
            if w > needed_w {
                needed_w = w;
            }
        }

        // Clamp width to usable screen area
        let max_w = screen_width() - 2.0 * MIN_INSET;
        let menu_w = needed_w.min(max_w);
        
        // Height depends on number of entries
        let menu_h = (shown.len() as f32) * ENTRY_H + 10.0;

        // Horizontal position: shift left if it would overflow the right edge
        let mut menu_x = button_rect.x;
        if menu_x + menu_w > screen_width() - MIN_INSET {
            menu_x = screen_width() - MIN_INSET - menu_w;
        }
        if menu_x < MIN_INSET {
            menu_x = MIN_INSET;
        }
        // Vertical position: just directly below the button
        let menu_y = button_rect.y + button_rect.h + MIN_INSET;
        let menu_rect = self.register_rect(Rect::new(menu_x, menu_y, menu_w, menu_h));

        // Background & border
        draw_rectangle(
            menu_rect.x,
            menu_rect.y,
            menu_rect.w,
            menu_rect.h,
            Color::new(0.0, 0.0, 0.0, 0.8),
        );
        draw_rectangle_lines(menu_rect.x, menu_rect.y, menu_rect.w, menu_rect.h, 2.0, WHITE);

        // Entries
        for (idx, reg) in shown.iter().enumerate() {
            let entry_rect = Rect::new(
                menu_rect.x + 5.0,
                menu_rect.y + 5.0 + idx as f32 * ENTRY_H,
                menu_rect.w - 10.0,
                25.0,
            );
            if gui_button(entry_rect, reg.type_name) {
                self.pending_add = Some(reg.type_name.to_string());
                self.add_mode = false;
            }
        }
    }

    /// Returns true if the currently selected entity can receive at least one
    /// component that is not already present
    fn can_show_any_component(&self, world_ecs: &WorldEcs) -> bool {
        let entity = match self.target {
            Some(e) => e,
            None => return false,
        };
        for entry in MODULES.iter() {
            let type_name = entry.title;
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == type_name) {
                if !entity_has_component(world_ecs, entity, reg) {
                    return true;
                }
            }
        }
        false
    }

    #[inline]
    fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }

    fn is_mouse_over(&self, mouse_screen: Vec2) -> bool {
        self.active_rects.iter().any(|r| r.contains(mouse_screen))
        || (self.rect.contains(mouse_screen) && self.target.is_some())
    }

    /// Checks whether the inspector was clicked during this frame.
    pub fn was_clicked(&self, mouse_screen: Vec2) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && self.is_mouse_over(mouse_screen)
    }
}

/// Utility function used by both the panel and the menu
fn entity_has_component(
    world_ecs: &WorldEcs,
    entity: Entity,
    reg: &ComponentReg,
) -> bool {
    (reg.has)(world_ecs, entity)
}