// editor/src/gui/inspector/player_module.rs
use engine_core::{ecs::component::Player, ui::widgets::*};
use macroquad::prelude::*;
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::{
        entity::Entity, 
        module::InspectorModule, 
        world_ecs::WorldEcs
    }
};

#[derive(Default)]
pub struct PlayerModule {}

impl InspectorModule for PlayerModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Player>(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        _asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        if let Some(_player) = world_ecs.get::<Player>(entity) {
            draw_text("Player Entity", rect.x, rect.y + 20.0, 18.0, FIELD_TEXT_COLOR);
        }
    }

    fn height(&self) -> f32 {
        20.0
    }

    fn title(&self) -> &str {
        std::any::type_name::<Self>()
            .rsplit("::")
            .next()
            .unwrap_or("Player")
    }
}