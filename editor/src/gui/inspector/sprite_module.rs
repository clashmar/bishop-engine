// editor/src/gui/inspector/sprite_module.rs
use macroquad::prelude::*;
use engine_core::{
    assets::{
        asset_manager::AssetManager, 
        sprite::Sprite
    }, 
    ecs::{
        entity::Entity, 
        module::{CollapsibleModule, InspectorModule}, 
        module_factory::ModuleFactoryEntry, 
        world_ecs::WorldEcs
    }, ui::widgets::*
};

#[derive(Default)]
pub struct SpriteModule {}

impl InspectorModule for SpriteModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Sprite>(entity).is_some()
    }

    fn removable(&self) -> bool { true }

    fn remove(&mut self, world_ecs: &mut WorldEcs, entity: Entity) {
        world_ecs.get_store_mut::<Sprite>().remove(entity);
    }

    fn draw(
        &mut self,
        rect: Rect,
        assets: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        let sprite = world_ecs
            .get_mut::<Sprite>(entity)
            .expect("Sprite must exist");

        let margin = 10.0;
        const LABEL: &str = "Choose Sprite";
        let txt = measure_text(LABEL, None, 20, 1.0);
        let btn_w = txt.width + 12.0;   
        let btn_h = txt.height + 8.0;

        // Center the button horizontally in the whole module
        let btn_x = rect.x + (rect.w - btn_w) / 2.0;

        // Keep a small top margin so the button isn’t glued to the header
        let btn_y = rect.y + margin;
        let btn_rect = Rect::new(btn_x, btn_y, btn_w, btn_h);

        // Button press 
        if gui_button(btn_rect, LABEL) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG images", &["png"])
                .pick_file()
            {
                let path_str = path.to_string_lossy().into_owned();

                // Update the component
                sprite.path = path_str.clone();

                // Load (or reuse) the texture and store the new id
                let new_id = futures::executor::block_on(assets.load(&path_str));
                sprite.sprite_id = new_id;
            }
        }
    }

    fn height(&self) -> f32 {
        40.0
    }
}

inventory::submit! {
    ModuleFactoryEntry {
        title: <engine_core::assets::sprite::Sprite>::TYPE_NAME,
        factory: || {
            Box::new(
                CollapsibleModule::new(
                    crate::gui::inspector::sprite_module::SpriteModule::default()
                )
                .with_title(<engine_core::assets::sprite::Sprite>::TYPE_NAME)
            )
        },
    }
}