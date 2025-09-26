use engine_core::{
    animation::animation_clip::{
        Animation, 
        Clip, 
        ClipId, 
        ClipState
    }, 
    assets::{
        asset_manager::AssetManager, 
        sprite::SpriteId
    }, 
    constants::TILE_SIZE, 
    ecs::{
        entity::Entity, 
        module::{CollapsibleModule, InspectorModule}, 
        module_factory::ModuleFactoryEntry, 
        world_ecs::WorldEcs
    }, 
    ui::widgets::*
};
use macroquad::prelude::*;
use uuid::Uuid;
use crate::gui::gui_constants::*;

#[derive(Default)]
pub struct AnimationModule {}

impl InspectorModule for AnimationModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Animation>(entity).is_some()
    }

    fn removable(&self) -> bool { true }

    fn remove(&mut self, world_ecs: &mut WorldEcs, entity: Entity) {
        world_ecs.get_store_mut::<Animation>().remove(entity);
    }

    fn draw(
        &mut self,
        rect: Rect,
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        let animation = world_ecs
            .get_mut::<Animation>(entity)
            .expect("Animation must exist");

        let mut y = rect.y + SPACING;
        let full_w = rect.w - 2.0 * PADDING;

        // Clip selector 
        // TODO: CREATE DROPDOWN WIDGET
        let clip_btn = Rect::new(rect.x + PADDING, y, full_w, BTN_HEIGHT);
        if gui_button(clip_btn,
            format!("{:?}", animation.current).as_str()) {
            // Cycle through keys
            let keys: Vec<_> = animation.clips.keys().cloned().collect();
            if let Some(idx) = keys.iter().position(|k| *k == animation.current) {
                let next = keys[(idx + 1) % keys.len()].clone();
                animation.set_clip(next);
            }
        }
        y += MARGIN + PADDING;
        
        // Add clip button
        const ADD_LABEL: &str = "Add Clip";
        let txt = measure_text(ADD_LABEL, None, 20, 1.0);
        let btn_w = txt.width + 12.0;   
        let btn_h = txt.height + 8.0;

        // Center the button horizontally in the whole module
        let btn_x = rect.x + (rect.w - btn_w) / 2.0;

        let btn_rect = Rect::new(btn_x, y, btn_w, btn_h);

        // Button press 
        if gui_button(btn_rect, ADD_LABEL) {
            let new_id = ClipId::Custom(format!("Clip {}", animation.clips.len() + 1));
            animation.clips.insert(new_id.clone(),
                Clip {
                    sprite_id: SpriteId(Uuid::nil()),
                    frame_size: vec2(TILE_SIZE, TILE_SIZE),
                    cols: 4,
                    rows: 1,
                    fps: 8.0,
                    looping: true,
                    offset: Vec2::ZERO,
                });
            animation.states.insert(new_id.clone(), ClipState::default());
            animation.current = new_id;
        }

        y += MARGIN + PADDING;

        // Edit the currently selected clip
        if let Some(clip) = animation.clips.get_mut(&animation.current) {
            // Sprite picker
            let sprite_btn = Rect::new(rect.x + PADDING, y, full_w, MARGIN);
            if gui_button(sprite_btn,
                if clip.sprite_id.0.is_nil() { "Pick Sprite" } else { "Change Sprite" }) {
                    if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG images", &["png"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().into_owned();

                    // Load (or reuse) the texture and store the new id
                    let new_id = futures::executor::block_on(asset_manager.load(&path_str));
                    clip.sprite_id = new_id;
                }
            }

            y += MARGIN + PADDING;

            // Frame size
            let w_input = Rect::new(rect.x + PADDING, y, full_w * 0.45, MARGIN);
            let h_input = Rect::new(rect.x + PADDING + full_w * 0.5, y, full_w * 0.45, MARGIN);
            clip.frame_size.x = gui_input_number(w_input, clip.frame_size.x);
            clip.frame_size.y = gui_input_number(h_input, clip.frame_size.y);
            y += MARGIN + PADDING;

            // Columns / rows
            let cols_input = Rect::new(rect.x + PADDING, y, full_w * 0.45, MARGIN);
            let rows_input = Rect::new(rect.x + PADDING + full_w * 0.5, y, full_w * 0.45, MARGIN);
            clip.cols = gui_input_number(cols_input, clip.cols as f32) as usize;
            clip.rows = gui_input_number(rows_input, clip.rows as f32) as usize;
            y += MARGIN + PADDING;

            // FPS
            let fps_input = Rect::new(rect.x + PADDING, y, full_w, MARGIN);
            clip.fps = gui_input_number(fps_input, clip.fps);
            y += MARGIN + PADDING;

            // Loop toggle
            let loop_btn = Rect::new(rect.x + PADDING, y, full_w, MARGIN);
            if gui_button(loop_btn,
                if clip.looping { "Loop: x" } else { "Loop: [ ]" }) {
                clip.looping = !clip.looping;
            }
            y += MARGIN + PADDING;

            // Optional offset
            let off_x = Rect::new(rect.x + PADDING, y, full_w * 0.45, MARGIN);
            let off_y = Rect::new(rect.x + PADDING + full_w * 0.5, y, full_w * 0.45, MARGIN);
            clip.offset.x = gui_input_number(off_x, clip.offset.x);
            clip.offset.y = gui_input_number(off_y, clip.offset.y);
        }
    }

    fn height(&self) -> f32 {
        400.0
    }
}

inventory::submit! {
    ModuleFactoryEntry {
        title: <engine_core::animation::animation_clip::Animation>::TYPE_NAME,
        factory: || {
            Box::new(
                CollapsibleModule::new(
                    crate::gui::inspector::animation_module::AnimationModule::default()
                )
                .with_title(<engine_core::animation::animation_clip::Animation>::TYPE_NAME)
            )
        },
    }
}