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

// Width of a three‑digit numeric field
const NUM_FIELD_W: f32 = 40.0;
const LABEL_Y_OFFSET: f32 = 5.0;
const COLON_GAP: f32 = 0.0;
const FIELD_GAP: f32 = 20.0;

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
                    path: String::new(),
                    frame_size: vec2(TILE_SIZE, TILE_SIZE),
                    cols: 4,
                    rows: 1,
                    fps: 8.0,
                    looping: true,
                    offset: Vec2::ZERO,
                });
            animation.states.insert(new_id.clone(), ClipState::default());
            animation.current = Some(new_id);
        }

        y += MARGIN + PADDING;

        // Return if there is no current id
        let Some(current_id) = &animation.current else { return };

        // Edit the currently selected clip
        if let Some(clip) = animation.clips.get_mut(&current_id) {
            // Clip selector 
            let clip_btn = Rect::new(rect.x + PADDING, y, full_w, BTN_HEIGHT);
            if gui_button(clip_btn,
                format!("{:?}", animation.current).as_str()) {
                // TODO: Make dropdown widget
            }
            
            y += MARGIN + PADDING;

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
                    clip.path = path_str;
                }
            }

            y += MARGIN + PADDING;

            // Frame size
            draw_frame_size_fields(y, rect, clip);
            y += MARGIN + PADDING;

            // Columns / rows
            draw_spritesheet_dimension_fields(y, rect, clip);
            y += MARGIN + PADDING;

            // FPS / Loop toggle
            draw_fps_and_loop(y, rect, clip);
            y += MARGIN + PADDING;

            // Optional offset
            draw_offset_fields(y, rect, clip);
        }
    }

    fn height(&self) -> f32 {
        400.0
    }
}

pub fn draw_frame_size_fields(y: f32, rect: Rect, clip: &mut Clip) {
    const LABEL_X: &str = "Frame X:";
    const LABEL_Y: &str = "Frame Y:";

    let width_x = measure_text(LABEL_X, None, 20, 1.0).width + COLON_GAP;
    let width_y = measure_text(LABEL_Y, None, 20, 1.0).width + COLON_GAP;

    let frame_x_label = Rect::new(
        rect.x + PADDING, 
        y + LABEL_Y_OFFSET, 
        width_x, 
        INPUT_HEIGHT
    );
    let frame_x_input = Rect::new(
        frame_x_label.x + width_x,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );
    let frame_y_label = Rect::new(
        frame_x_input.x + NUM_FIELD_W + FIELD_GAP,
        y + LABEL_Y_OFFSET,
        width_y,
        INPUT_HEIGHT,
    );
    let frame_y_input = Rect::new(
        frame_y_label.x + width_y,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );

    // Render labels
    draw_text(LABEL_X, frame_x_label.x, frame_x_label.y + 15.0, 18.0, WHITE);
    draw_text(LABEL_Y, frame_y_label.x, frame_y_label.y + 15.0, 18.0, WHITE);

    // Numeric inputs
    clip.frame_size.x = gui_input_number(frame_x_input, clip.frame_size.x);
    clip.frame_size.y = gui_input_number(frame_y_input, clip.frame_size.y);
}

pub fn draw_spritesheet_dimension_fields(y: f32, rect: Rect, clip: &mut Clip) {
    const LABEL_X: &str = "Cols:";
    const LABEL_Y: &str = "Rows:";

    let width_x = measure_text(LABEL_X, None, 20, 1.0).width + COLON_GAP;
    let width_y = measure_text(LABEL_Y, None, 20, 1.0).width + COLON_GAP;

    let frame_x_label = Rect::new(
        rect.x + PADDING, 
        y + LABEL_Y_OFFSET, 
        width_x, 
        INPUT_HEIGHT
    );
    let frame_x_input = Rect::new(
        frame_x_label.x + width_x,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );
    let frame_y_label = Rect::new(
        frame_x_input.x + NUM_FIELD_W + FIELD_GAP,
        y + LABEL_Y_OFFSET,
        width_y,
        INPUT_HEIGHT,
    );
    let frame_y_input = Rect::new(
        frame_y_label.x + width_y,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );

    // Render labels
    draw_text(LABEL_X, frame_x_label.x, frame_x_label.y + 15.0, 18.0, WHITE);
    draw_text(LABEL_Y, frame_y_label.x, frame_y_label.y + 15.0, 18.0, WHITE);

    // Numeric inputs
    clip.cols = gui_input_number(frame_x_input, clip.cols as f32) as usize;
    clip.rows = gui_input_number(frame_y_input, clip.rows as f32) as usize;
}

pub fn draw_fps_and_loop(y: f32, rect: Rect, clip: &mut Clip) {
    const LABEL_X: &str = "FPS:";
    const LABEL_Y: &str = "Loop:";

    let width_x = measure_text(LABEL_X, None, 20, 1.0).width + COLON_GAP;
    let width_y = measure_text(LABEL_Y, None, 20, 1.0).width + COLON_GAP;

    let frame_x_label = Rect::new(
        rect.x + PADDING, 
        y + LABEL_Y_OFFSET, 
        width_x, 
        INPUT_HEIGHT
    );
    let frame_x_input = Rect::new(
        frame_x_label.x + width_x,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );
    let frame_y_label = Rect::new(
        frame_x_input.x + NUM_FIELD_W + FIELD_GAP,
        y + LABEL_Y_OFFSET,
        width_y,
        INPUT_HEIGHT,
    );
    let frame_y_input = Rect::new(
        frame_y_label.x + width_y,
        y + LABEL_Y_OFFSET,
        CHECKBOX_SIZE,
        CHECKBOX_SIZE,
    );

    // Render labels
    draw_text(LABEL_X, frame_x_label.x, frame_x_label.y + 15.0, 18.0, WHITE);
    draw_text(LABEL_Y, frame_y_label.x, frame_y_label.y + 15.0, 18.0, WHITE);

    // Numeric inputs
    clip.fps = gui_input_number(frame_x_input, clip.fps);
    if gui_checkbox(frame_y_input, &mut clip.looping) {
        clip.looping = clip.looping
    }
}

pub fn draw_offset_fields(y: f32, rect: Rect, clip: &mut Clip) {
    const LABEL_X: &str = "Offset X:";
    const LABEL_Y: &str = "Offset Y:";

    let width_x = measure_text(LABEL_X, None, 20, 1.0).width + COLON_GAP;
    let width_y = measure_text(LABEL_Y, None, 20, 1.0).width + COLON_GAP;

    let frame_x_label = Rect::new(
        rect.x + PADDING, 
        y + LABEL_Y_OFFSET, 
        width_x, 
        INPUT_HEIGHT
    );
    let frame_x_input = Rect::new(
        frame_x_label.x + width_x,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );
    let frame_y_label = Rect::new(
        frame_x_input.x + NUM_FIELD_W + FIELD_GAP,
        y + LABEL_Y_OFFSET,
        width_y,
        INPUT_HEIGHT,
    );
    let frame_y_input = Rect::new(
        frame_y_label.x + width_y,
        y,
        NUM_FIELD_W,
        INPUT_HEIGHT,
    );

    // Render labels
    draw_text(LABEL_X, frame_x_label.x, frame_x_label.y + 15.0, 18.0, WHITE);
    draw_text(LABEL_Y, frame_y_label.x, frame_y_label.y + 15.0, 18.0, WHITE);

    // Numeric inputs
    clip.offset.x = gui_input_number(frame_x_input, clip.offset.x);
    clip.offset.y = gui_input_number(frame_y_input, clip.offset.y);
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