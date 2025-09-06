use core::{assets::asset_manager::AssetManager, ecs::{entity::Entity, world_ecs::WorldEcs}};
use crate::gui::{gui_button, inspector::{
    module::InspectorModule, 
    sprite::SpriteModule, 
    transform::TransformModule
}};
use macroquad::prelude::*;

/// The panel that lives on the right‑hand side of the room editor window.
pub struct InspectorPanel {
    /// Geometry of the panel.
    rect: Rect,
    /// Currently inspected entity
    target: Option<Entity>,
    /// All sub‑modules that can draw UI.
    modules: Vec<Box<dyn InspectorModule>>,
}

impl InspectorPanel {
    /// Create a fresh panel with the default set of modules.
    pub fn new() -> Self {
        let mut modules: Vec<Box<dyn InspectorModule>> = Vec::new();
        modules.push(Box::new(TransformModule::default()));
        modules.push(Box::new(SpriteModule::default()));

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

        // No entity selected
        if self.target.is_none() {
            println!("here");
            let btn = Rect::new(
                inner.x + inner.w - BTN_W - BTN_MARGIN, 
                inner.y + BTN_MARGIN,                 
                BTN_W,
                BTN_H,
            );

            // No background is drawn in this state – the button is the only
            // visible UI element.
            return gui_button(btn, "Create");
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
                let sub_rect = Rect::new(inner.x + 10.0, y, inner.w - 20.0, 80.0);
                module.draw(sub_rect, assets, ecs, entity);
                y += sub_rect.h + 10.0;
            }
        }

        false
    }
}