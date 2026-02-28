// editor/src/gui/inspector/player_module.rs
use engine_core::prelude::*;
use bishop::prelude::*;

#[derive(Default)]
pub struct PlayerModule {}

impl InspectorModule for PlayerModule {
    fn visible(&self, ecs: &Ecs, entity: Entity) -> bool {
        ecs.has::<Player>(entity) || ecs.has::<PlayerProxy>(entity)
    }

    fn draw(
        &mut self,
        ctx: &mut WgpuContext,
        _blocked: bool,
        rect: Rect,
        game_ctx: &mut GameCtxMut,
        entity: Entity,
    ) {
        let ecs = &game_ctx.ecs;

        if ecs.has::<Player>(entity) {
            ctx.draw_text("Player Entity", rect.x, rect.y + 20.0, 18.0, FIELD_TEXT_COLOR);
        } else if ecs.has::<PlayerProxy>(entity) {
            ctx.draw_text("Player Proxy", rect.x, rect.y + 20.0, 18.0, FIELD_TEXT_COLOR);
        }
    }

    fn height(&self) -> f32 {
        28.0
    }

    fn title(&self) -> &str {
        std::any::type_name::<Self>()
            .rsplit("::")
            .next()
            .unwrap_or("Player")
    }
}