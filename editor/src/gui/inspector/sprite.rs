// editor/src/gui/inspector/sprite.rs
use macroquad::prelude::*;
use crate::gui::inspector::module::InspectorModule;
use engine_core::{
    assets::{
        asset_manager::AssetManager, sprite::Sprite
    }, 
    ecs::{
        entity::Entity, world_ecs::WorldEcs
    }
};

#[derive(Default)]
pub struct SpriteModule {}

impl InspectorModule for SpriteModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Sprite>(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        _asset_manager: &mut AssetManager,
        _world_ecs: &mut WorldEcs,
        _entity: Entity,
    ) {
        draw_text(
            "Sprite (placeholder)",
            rect.x + 5.0,
            rect.y + 20.0,
            20.0,
            WHITE,
        );
    }
}