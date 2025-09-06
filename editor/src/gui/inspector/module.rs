use macroquad::prelude::*;
use core::assets::asset_manager::AssetManager;
use core::ecs::world_ecs::WorldEcs;
use core::ecs::entity::Entity;

/// Every inspector sub‑module implements this trait.
pub trait InspectorModule {
    /// Return true when the module should be shown for the given entity.
    fn visible(&self, ecs: &WorldEcs, entity: Entity) -> bool;

    /// Draw the UI for the module inside the supplied rectangle.
    fn draw(
        &mut self,
        rect: Rect,
        assets: &mut AssetManager,
        ecs: &mut WorldEcs,
        entity: Entity,
    );
}