// editor/src/gui/inspector/panel.rs
use macroquad::prelude::*;
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::{
        component::Weapon, 
        entity::Entity, 
        world_ecs::WorldEcs
    }
};
use crate::gui::{
    gui_button, 
    inspector::{
        generic::GenericModule, 
        module::{CollapsibleModule, InspectorModule}, 
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

        modules.push(Box::new(CollapsibleModule::new(GenericModule::<Weapon>::default())
            .with_title("Weapon")));

        Self {
            rect: Rect::new(0., 0., 0., 0.),
            target: None,
            modules,
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
    pub fn draw(&mut self, assets: &mut AssetManager, ecs: &mut WorldEcs) -> bool {

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
            if module.visible(ecs, entity) {
                let h = module.height();  // dynamic height                     
                let sub_rect = Rect::new(inner.x + 10.0, y, inner.w - 20.0, h);
                module.draw(sub_rect, assets, ecs, entity);
                y += h + 10.0; // space between modules                               
            }
        }

        false
    }
}