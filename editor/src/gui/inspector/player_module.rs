// editor/src/gui/inspector/player_module.rs
use engine_core::ecs::entity::Entity;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::ecs::module::InspectorModule;
use engine_core::{ecs::component::Player, game::game::GameCtx, ui::{text::*, widgets::*}};
use macroquad::prelude::*;

#[derive(Default)]
pub struct PlayerModule {}

impl InspectorModule for PlayerModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Player>(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        game_ctx: &mut GameCtx,
        entity: Entity,
    ) {
        let world_ecs = &mut game_ctx.cur_world_ecs;

        if let Some(_player) = world_ecs.get::<Player>(entity) {
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