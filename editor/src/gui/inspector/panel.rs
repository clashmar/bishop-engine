// editor/src/gui/inspector/panel.rs
use macroquad::prelude::*;
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::{
        component_registry::{COMPONENTS, ComponentReg}, 
        entity::Entity, 
        module::{CollapsibleModule, InspectorModule}, 
        module_factory::{MODULES, ModuleFactoryEntry}, 
        world_ecs::WorldEcs
    }
};
use crate::gui::{
    gui_button, 
    inspector::{
        transform::TransformModule
    }
};

/// The panel that lives on the right‑hand side of the room editor window.
pub struct InspectorPanel {
    /// Geometry of the panel.
    rect: Rect,
    /// Currently inspected entity
    pub target: Option<Entity>,
    /// All sub‑modules that can draw UI.
    modules: Vec<Box<dyn InspectorModule>>,
    show_add_menu: bool,
    pending_add: Option<String>,

}

impl InspectorPanel {
    /// Create a fresh panel with the default set of modules.
    pub fn new() -> Self {
        let mut modules: Vec<Box<dyn InspectorModule>> = Vec::new();
        // Wrap each concrete module in a CollapsibleModule.
        modules.push(Box::new(
            CollapsibleModule::new(TransformModule::default())
                .with_title("Transform"),
        ));

        // Add generic inmodules here
        for entry in MODULES.iter() {
            modules.push((entry.creator)());
        }

        Self {
            rect: Rect::new(0., 0., 0., 0.),
            target: None,
            modules,
            show_add_menu: false,
            pending_add: None,
        }
    }

    /// Called by the editor each frame to place the panel.
    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    /// Tell the inspector which entity is currently selected.
    pub fn set_target(&mut self, entity: Option<Entity>) {
        self.target = entity;
    }

    /// Render the panel and any visible sub‑modules.
    /// Returns true if 'Create' was pressed.
    pub fn draw(
        &mut self, 
        assets: &mut AssetManager, 
        world_ecs: &mut WorldEcs,
    ) -> bool {

        const INSET: f32 = 10.0;          
        const BTN_W: f32 = 80.0;          
        const BTN_H: f32 = 30.0;         
        const BTN_MARGIN: f32 = 10.0; 
        
        // The geometry of the panel
        let inner = Rect::new(
            self.rect.x,                     
            self.rect.y + INSET,            
            self.rect.w - INSET,             
            self.rect.h - INSET * 2.0,    
        );

        // No entity selected, draw create button
        if self.target.is_none() {
            let button = Rect::new(
                inner.x + inner.w - BTN_W - BTN_MARGIN, 
                inner.y + BTN_MARGIN,                 
                BTN_W,
                BTN_H,
            );
            return gui_button(button, "Create");
        }

        // Background
        draw_rectangle(
            inner.x,
            inner.y,
            inner.w,
            inner.h,
            Color::new(0., 0., 0., 0.6),
        );

        // Outline
        draw_rectangle_lines(
            inner.x, 
            inner.y, 
            inner.w, 
            inner.h, 2., 
            WHITE
        );

        // Layout the modules vertically.
        let mut y = inner.y + 10.0; // start a bit below the top edge
        let entity = self.target.unwrap();

        for module in &mut self.modules {
            if module.visible(world_ecs, entity) {
                let h = module.height();  // dynamic height                     
                let sub_rect = Rect::new(inner.x + 10.0, y, inner.w - 20.0, h);
                module.draw(sub_rect, assets, world_ecs, entity);
                y += h + 10.0; // space between modules                               
            }
        }

        self.draw_add_component(world_ecs);

        // Process pending requests
        if let (Some(name), Some(entity)) = (self.pending_add.take(), self.target) {
            // Find the registration that matches the name
            if let Some(reg) = COMPONENTS.iter().find(|r| r.type_name == name) {
                (reg.factory)(world_ecs, entity);
            } else {
                eprintln!("Component `{}` not found in registry", name);
            }
        }
        false
    }

    fn draw_add_component(&mut self, world_ecs: &mut WorldEcs) {
        let add_btn_rect = Rect::new(
            self.rect.x + 10.0,
            self.rect.y + self.rect.h - 40.0,
            self.rect.w - 20.0,
            30.0,
        );
        if gui_button(add_btn_rect, "Add Component") {
            self.show_add_menu = true;
        }

        if self.show_add_menu {
            let entity = match self.target {
                Some(e) => e,
                None => return,
            };

            // Collect the registrations that are actually shown.
            let mut shown: Vec<&ComponentReg> = Vec::new();
            for reg in COMPONENTS.iter() {
                if reg.type_name == "TileSprite" {
                    continue;
                }

                if entity_has_component(world_ecs, entity, reg) {
                    continue;
                }
                shown.push(reg);
            }

            let entry_h = 30.0;
            let menu_w = 200.0;
            let menu_h = (shown.len() as f32) * entry_h + 10.0;
            let menu_rect = Rect::new(
                add_btn_rect.x,
                add_btn_rect.y - menu_h,
                menu_w,
                menu_h,
            );

            // Background & border
            draw_rectangle(
                menu_rect.x,
                menu_rect.y,
                menu_rect.w,
                menu_rect.h,
                Color::new(0.0, 0.0, 0.0, 0.8),
            );
            draw_rectangle_lines(menu_rect.x, menu_rect.y, menu_rect.w, menu_rect.h, 2.0, WHITE);

            for (idx, reg) in shown.iter().enumerate() {
                let entry_rect = Rect::new(
                    menu_rect.x + 5.0,
                    menu_rect.y + 5.0 + idx as f32 * entry_h,
                    menu_rect.w - 10.0,
                    25.0,
                );
                if gui_button(entry_rect, reg.type_name) {
                    // User clicked a component name
                    self.pending_add = Some(reg.type_name.to_string());
                    self.show_add_menu = false;
                }
            }
        }
    }
}

fn entity_has_component(
        world_ecs: &WorldEcs,
        entity: Entity,
        reg: &ComponentReg,
    ) -> bool {
        (reg.has)(world_ecs, entity)
    }