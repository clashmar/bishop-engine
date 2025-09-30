// editor/src/gui/inspector/transform_module.rs
use macroquad::prelude::*;
use engine_core::ui::widgets::*;
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::{
        component::Position, 
        entity::Entity, 
        module::InspectorModule, 
        world_ecs::WorldEcs
    }
};

#[derive(Default)]
pub struct TransformModule {
    pub x_id: WidgetId,
    pub y_id: WidgetId,
}

impl TransformModule {
    /// Draw the two numeric fields that edit the position
    fn draw_position_fields(
        &self,
        rect: Rect,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        let pos = world_ecs.get_mut::<Position>(entity).expect("Position must exist");

        // Layout constants
        let label_w = 20.0;
        // Reduce the usable width
        let usable_w = rect.w * 0.9;
        let field_w = (usable_w - label_w - 10.0) / 2.0;
        let field_h = 30.0;
        let spacing = 5.0;

        // X
        let x_label = Rect::new(rect.x, rect.y, label_w, field_h);
        draw_text("X:", x_label.x + 2.0, x_label.y + 22.0, 18.0, WHITE);
        let x_field = Rect::new(
            x_label.x + label_w + spacing,
            rect.y,
            field_w,
            field_h,
        );
        let new_x = gui_input_number(self.x_id, x_field, pos.position.x);

        // Y
        let y_label = Rect::new(
            x_field.x + field_w + spacing,
            rect.y,
            label_w,
            field_h,
        );
        draw_text("Y:", y_label.x + 2.0, y_label.y + 22.0, 18.0, WHITE);
        let y_field = Rect::new(
            y_label.x + label_w + spacing,
            rect.y,
            field_w,
            field_h,
        );
        let new_y = gui_input_number(self.y_id, y_field, pos.position.y);

        // Write back only if something changed
        if (new_x - pos.position.x).abs() > f32::EPSILON
            || (new_y - pos.position.y).abs() > f32::EPSILON
        {
            pos.position.x = new_x;
            pos.position.y = new_y;
        }
    }
}

impl InspectorModule for TransformModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Position>(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        _asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        // Show the current world position
        if let Some(_pos) = world_ecs.get::<Position>(entity) {
            let readout = format!("World position:");
            draw_text(&readout, rect.x, rect.y + 20.0, 18.0, LIGHTGRAY);
        }

        // Editable numeric fields (X / Y)
        let edit_rect = Rect::new(
            rect.x,
            rect.y + 30.0,
            rect.w,
            40.0,
        );
        self.draw_position_fields(edit_rect, world_ecs, entity);
    }
}