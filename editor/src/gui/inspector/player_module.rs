// editor/src/gui/inspector/player_module.rs
use engine_core::{ecs::component::Player, game::game::GameCtxMut, ui::{text::*, widgets::*}};
use engine_core::ecs::inpsector_module::InspectorModule;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::ecs::Ecs;
use macroquad::prelude::*;

#[derive(Default)]
pub struct PlayerModule {}

impl InspectorModule for PlayerModule {
    fn visible(&self, ecs: &Ecs, entity: Entity) -> bool {
        ecs.get::<Player>(entity).is_some()
    }

    fn draw(
        &mut self,
        _blocked: bool,
        rect: Rect,
        game_ctx: &mut GameCtxMut,
        entity: Entity,
    ) {
        let ecs = &mut game_ctx.ecs;

        if let Some(_player) = ecs.get::<Player>(entity) {
            draw_text_ui("Player Entity", rect.x, rect.y + 20.0, 18.0, FIELD_TEXT_COLOR);
        }
    }

    fn height(&self) -> f32 {
        25.0
    }

    fn title(&self) -> &str {
        std::any::type_name::<Self>()
            .rsplit("::")
            .next()
            .unwrap_or("Player")
    }
}