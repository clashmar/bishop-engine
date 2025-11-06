use engine_core::{ecs::world_ecs::WorldEcs, world::
    room::Room
};
use macroquad::prelude::*;

pub trait DynamicTilemapUiElement {
    fn draw(&self, camera: &Camera2D);
    fn is_mouse_over(&self, mouse_pos: Vec2, camera: &Camera2D) -> bool;
    fn on_click(
        &mut self,
        room: &mut Room,
        mouse_pos: Vec2, 
        camera: &Camera2D,
        other_bounds: &[(Vec2, Vec2)],
        world_ecs: &mut WorldEcs,
    );
}


