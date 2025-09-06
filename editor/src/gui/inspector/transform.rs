use core::{assets::asset_manager::AssetManager, ecs::{entity::Entity, world_ecs::WorldEcs}};
use macroquad::prelude::*;
use crate::gui::inspector::module::InspectorModule;

#[derive(Default)]
pub struct TransformModule {}

impl InspectorModule for TransformModule {
    fn visible(&self, ecs: &WorldEcs, entity: Entity) -> bool {
        ecs.positions.get(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        _assets: &mut AssetManager,
        _ecs: &mut WorldEcs,
        _entity: Entity,
    ) {
        draw_text(
            "Transform (placeholder)",
            rect.x + 5.0,
            rect.y + 20.0,
            20.0,
            WHITE,
        );
    }
}