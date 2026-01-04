// editor/src/gui/inspector/transform_module.rs
use engine_core::ecs::inpsector_module::InspectorModule;
use engine_core::ecs::position::Position;
use engine_core::game::game::GameCtxMut;
use engine_core::ecs::entity::Entity;
use engine_core::ui::widgets::*;
use engine_core::ecs::ecs::Ecs;
use engine_core::ui::text::*;
use macroquad::prelude::*;

#[derive(Default)]
pub struct TransformModule {
    pub x_id: WidgetId,
    pub y_id: WidgetId,
}

// TODO: Add rotation
impl TransformModule {
    /// Draw the two numeric fields that edit the position
    fn draw_position_fields(
        &self,
        blocked: bool,
        rect: Rect,
        ecs: &mut Ecs,
        entity: Entity,
    ) {
        let pos = ecs.get_mut::<Position>(entity).expect("Position must exist.");

        // Layout constants
        let label_w = 20.0;
        // Reduce the usable width
        let usable_w = rect.w * 0.9;
        let field_w = (usable_w - label_w - 10.0) / 2.0;
        let field_h = 30.0;
        let spacing = 5.0;

        // X
        let x_label = Rect::new(rect.x, rect.y, label_w, field_h);
        draw_text_ui("X :", x_label.x + 2.0, x_label.y + 22.0, 18.0, FIELD_TEXT_COLOR);
        let x_field = Rect::new(
            x_label.x + label_w + spacing,
            rect.y,
            field_w,
            field_h,
        );
        let new_x = gui_input_number_f32(self.x_id, x_field, pos.position.x, blocked);

        // Y
        let y_label = Rect::new(
            x_field.x + field_w + spacing,
            rect.y,
            label_w,
            field_h,
        );
        draw_text_ui("Y :", y_label.x + 2.0, y_label.y + 22.0, 18.0, FIELD_TEXT_COLOR);
        let y_field = Rect::new(
            y_label.x + label_w + spacing,
            rect.y,
            field_w,
            field_h,
        );
        let new_y = gui_input_number_f32(self.y_id, y_field, pos.position.y, blocked);

        // Write back only if something changed
        if !blocked && (new_x - pos.position.x).abs() > f32::EPSILON
            || (new_y - pos.position.y).abs() > f32::EPSILON
        {
            pos.position.x = new_x;
            pos.position.y = new_y;
        }
    }
}

impl InspectorModule for TransformModule {
    fn visible(&self, ecs: &Ecs, entity: Entity) -> bool {
        ecs.get::<Position>(entity).is_some()
    }

    fn draw(
        &mut self,
        blocked: bool,
        rect: Rect,
        game_ctx: &mut GameCtxMut,
        entity: Entity,
    ) {
        let ecs = &mut game_ctx.ecs;

        // Show the current world position
        if let Some(_pos) = ecs.get::<Position>(entity) {
            let readout = format!("World position :");
            draw_text_ui(&readout, rect.x, rect.y + 20.0, 18.0, FIELD_TEXT_COLOR);
        }

        // Editable numeric fields (X / Y)
        let edit_rect = Rect::new(
            rect.x,
            rect.y + 30.0,
            rect.w,
            40.0,
        );
        self.draw_position_fields(blocked, edit_rect, ecs, entity);
    }
}