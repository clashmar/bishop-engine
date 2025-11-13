use std::{borrow::Cow, collections::{HashMap, HashSet}, path::Path};
use strum::IntoEnumIterator;
use engine_core::{
    animation::{
            animation_clip::{
            Animation, ClipDef, 
            ClipId, 
            ClipState, 
            VariantFolder
        }, 
        animation_system::CurrentFrame
    }, 
    assets::asset_manager::AssetManager, 
    ecs::{
        entity::Entity, 
        module::{CollapsibleModule, InspectorModule}, 
        module_factory::ModuleFactoryEntry, 
        world_ecs::WorldEcs
    }, 
    ui::{text::*, toast::Toast, widgets::*}
};
use macroquad::prelude::*;
use crate::gui::gui_constants::*;

// Width of a three‑digit numeric field
const NUM_FIELD_W: f32 = 40.0;
const LABEL_Y_OFFSET: f32 = 20.0;
const LABEL_FONT_SIZE: f32 = DEFAULT_FONT_SIZE;
const COLON_GAP: f32 = 10.0;
const FIELD_GAP: f32 = 20.0;

#[derive(Default)]   
pub struct AnimationModule {
    pending_rename: bool,
    rename_initial_value: String,
    warning: Option<Toast>,
    select_dropdown_id: WidgetId,
    set_dropdown_id: WidgetId,
    rename_field_id: WidgetId,
    frame_x_id: WidgetId,
    frame_y_id: WidgetId,
    cols_id: WidgetId,
    rows_id: WidgetId,
    fps_id: WidgetId,
    offset_x_id: WidgetId,
    offset_y_id: WidgetId,
}

impl InspectorModule for AnimationModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Animation>(entity).is_some()
    }

    fn removable(&self) -> bool { true }

    fn remove(&mut self, world_ecs: &mut WorldEcs, entity: Entity) {
        world_ecs.get_store_mut::<Animation>().remove(entity);
        world_ecs.get_store_mut::<CurrentFrame>().remove(entity);
    }

    fn draw(
        &mut self,
        rect: Rect,
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        let mut variant_changed = false;
        let mut all_ids: Vec<ClipId> = vec![];
        fill_all_clip_ids(&world_ecs, &mut all_ids);
        
        let animation = world_ecs
            .get_mut::<Animation>(entity)
            .expect("Animation must exist");

        let mut y = rect.y + SPACING;
        let full_w = rect.w - 2.0 * PADDING;

        // Add-clip button
        const ADD_LABEL: &str = "Add Clip";
        let txt = measure_text_ui(ADD_LABEL, DEFAULT_FONT_SIZE, 1.0);
        let btn_w = txt.width + 12.0;   
        let btn_h = txt.height + 8.0;

        // Center the button horizontally in the whole module
        let btn_x = rect.x + (rect.w - btn_w) / 2.0;
        let btn_rect = Rect::new(btn_x, y, btn_w, btn_h);

        // Button press
        if gui_button(btn_rect, ADD_LABEL) {
            let new_id = if animation.clips.is_empty() {
                ClipId::Idle
            } else {
                // All concrete ids that are not yet used
                let used: HashSet<_> = animation.clips.keys().cloned().collect();
                let next_builtin = ClipId::iter()
                    .filter(|id| !matches!(id, ClipId::New | ClipId::Custom(_)))
                    .find(|id| !used.contains(id));
                match next_builtin {
                    Some(id) => id.clone(),
                    None => ClipId::Custom(format!("New Clip {}", animation.clips.len() + 1)),
                }
            };
            animation.clips.insert(new_id.clone(), ClipDef::default());
            animation.states.insert(new_id.clone(), ClipState::default());
            animation.current = Some(new_id);
        }

        y += MARGIN + PADDING;
        
        // Return if there is no current id
        if animation.current.is_none() {
            return;
        }

        // Calculate clip selector dropdown here
        let clip_dropdown_rect = Rect::new(rect.x + PADDING, y, full_w, BTN_HEIGHT);
        
        y += MARGIN + PADDING;

        // Edit the currently selected clip
        if let Some(clip) = animation.clips.get_mut(&animation.current.as_ref().unwrap()) {
            // Variant picker
            let has_variant = !animation.variant.0.as_os_str().is_empty();

            let sprite_btn = Rect::new(rect.x + PADDING, y, full_w / 2., MARGIN);

            if gui_button(sprite_btn,
                if has_variant { "Edit Variant" } else { "Choose Variant" }) {
                if let Some(path) = rfd::FileDialog::new()
                    .pick_folder()
                {
                    let normalized_path = asset_manager.normalize_path(path);
                    animation.variant = VariantFolder(normalized_path);
                    variant_changed = true;
                }
            }

            let full_path = Path::new(&animation.variant.0);

            let variant_label = if has_variant {
                full_path
                    .file_name()
                    .map(|n| Cow::Owned(format!("/{}", n.to_string_lossy().into_owned())))
                    .unwrap_or_else(|| Cow::Borrowed("/..."))
            } else {
                Cow::Borrowed("/...")
            };

            draw_text_ui(
                &variant_label, 
                rect.x + sprite_btn.w + SPACING + PADDING, 
                y + LABEL_Y_OFFSET, 
                DEFAULT_FONT_SIZE, 
                FIELD_TEXT_COLOR
            );

            y += MARGIN + PADDING;

            // Frame size
            draw_frame_size_fields(self, y, rect, clip);
            y += MARGIN + PADDING;

            // Columns / rows
            draw_spritesheet_dimension_fields(self, y, rect, clip);
            y += MARGIN + PADDING;

            // FPS / Loop toggle
            draw_fps_and_loop(self, y, rect, clip);
            y += MARGIN + PADDING;

            // Optional offset
            draw_offset_fields(self, y, rect, clip);
        }

        draw_current_clip_dropdowns(
            self,
            clip_dropdown_rect, 
            animation, 
            all_ids,
        );

        if let Some(toast) = &mut self.warning {
            toast.update();
            if !toast.active {
                self.warning = None;
            }
        }
        if variant_changed {
            futures::executor::block_on(animation.refresh_sprite_cache(asset_manager));
        }
    }

    fn height(&self) -> f32 {
        400.0
    }
}

pub fn draw_current_clip_dropdowns(
    module: &mut AnimationModule,
    rect: Rect, 
    animation: &mut Animation, 
    all_ids: Vec<ClipId>,
) {
    let current_id = animation.current.as_ref().unwrap();
    let clip_label = format!("{current_id}");
    let width = rect.w / 2.0 - SPACING;
    // Select clip
    let select_rect = Rect::new(rect.x, rect.y, width, rect.h);

    if let Some(selected) = gui_dropdown(
        module.select_dropdown_id,
        select_rect,
        &clip_label,
        &existing_clip_ids(&animation.clips),
        |id| id.ui_label(),
    ) {
        animation.set_clip(&selected);
        return;
    }

    // Edit the ClipId of the current clip
    let right_rect = Rect::new(
        (select_rect.x + rect.w) - (width),
        rect.y,
        width,
        rect.h,
    );

    // Show the type selector
    let type_label = "Set Type";

    let chosen = gui_dropdown(
        module.set_dropdown_id,
        right_rect,
        &type_label,
        &all_ids,
        |id| id.ui_label(),
    );

    if let Some(chosen) = chosen {
        match chosen {
            // For now always open the rename field
            ClipId::New => {
                module.pending_rename = true;
                module.rename_initial_value.clear();          
                return;
            }
            ClipId::Custom(name) => {
                module.pending_rename = true;
                module.rename_initial_value = name.clone();   
                return;
            }
            // Any other enum variant
            other => {
                // Prevent duplicate concrete types on the same entity
                if animation.clips.contains_key(&other) && Some(&other) != animation.current.as_ref() {
                    module.warning = Some(Toast::new(
                        format!("Enity already has this animation."),
                        2.0, // seconds
                    ));
                } else {
                    reset_current_clip_id(animation, other);
                    module.pending_rename = false;
                }
                return;
            }
        }
    }

    // Render the rename text field while the flag is true
    if module.pending_rename {
        // Position directly under the right‑hand dropdown
        let input_rect = Rect::new(
            right_rect.x,
            right_rect.y + right_rect.h + 4.0,
            right_rect.w,
            INPUT_HEIGHT,
        );

        const CLAMP: usize = 12;

        // The field starts empty each time we open it
        let (entered, focused) = gui_input_text_clamped_focused(
            module.rename_field_id,
            input_rect, 
            &module.rename_initial_value, 
            CLAMP
        );

        // Check if enter is pressed first
        if is_key_pressed(KeyCode::Enter) {
            let new_id = ClipId::Custom(entered.trim().to_string());
            reset_current_clip_id(animation, new_id);
            module.pending_rename = false;
            gui_input_text_reset(module.rename_field_id);
        }
        else if !focused {
            gui_input_text_reset(module.rename_field_id);
            module.pending_rename = false;
        }
    }
}

pub fn draw_frame_size_fields(
    module: &mut AnimationModule,
    y: f32, 
    rect: Rect, 
    clip: &mut ClipDef
) {
    const LABELS: [&str; 2] = ["Frame X:", "Frame Y:"];
    let (lbl_x, inp_x, lbl_y, inp_y) = layout_pair(y, rect, LABELS);

    // Render the two labels
    draw_text_ui(LABELS[0], lbl_x.x, lbl_x.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);
    draw_text_ui(LABELS[1], lbl_y.x, lbl_y.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);

    // Numeric inputs
    clip.frame_size.x = gui_input_number_f32(module.frame_x_id, inp_x, clip.frame_size.x);
    clip.frame_size.y = gui_input_number_f32(module.frame_y_id, inp_y, clip.frame_size.y);
}

pub fn draw_spritesheet_dimension_fields(
    module: &mut AnimationModule,
    y: f32, 
    rect: Rect, 
    clip: &mut ClipDef
) {
    const LABELS: [&str; 2] = ["Cols:", "Rows:"];
    let (lbl_c, inp_c, lbl_r, inp_r) = layout_pair(y, rect, LABELS);

    draw_text_ui(LABELS[0], lbl_c.x, lbl_c.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);
    draw_text_ui(LABELS[1], lbl_r.x, lbl_r.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);

    clip.cols = gui_input_number_f32(module.cols_id, inp_c, clip.cols as f32) as usize;
    clip.rows = gui_input_number_f32(module.rows_id, inp_r, clip.rows as f32) as usize;
}

pub fn draw_fps_and_loop(
    module: &mut AnimationModule,
    y: f32, 
    rect: Rect, 
    clip: &mut ClipDef
) {
    const LABELS: [&str; 2] = ["FPS:", "Loop:"];
    let (lbl_fps, inp_fps, lbl_loop, mut inp_loop) = layout_pair(y, rect, LABELS);
    inp_loop.w = CHECKBOX_SIZE;
    inp_loop.h = CHECKBOX_SIZE;
    inp_loop.y += 5.;

    draw_text_ui(LABELS[0], lbl_fps.x, lbl_fps.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);
    draw_text_ui(LABELS[1], lbl_loop.x, lbl_loop.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);

    clip.fps = gui_input_number_f32(module.fps_id, inp_fps, clip.fps);
    gui_checkbox(inp_loop, &mut clip.looping);
}

pub fn draw_offset_fields(
    module: &mut AnimationModule,
    y: f32, 
    rect: Rect, 
    clip: &mut ClipDef
) {
    const LABELS: [&str; 2] = ["Offset X:", "Offset Y:"];
    let (lbl_x, inp_x, lbl_y, inp_y) = layout_pair(y, rect, LABELS);

    draw_text_ui(LABELS[0], lbl_x.x, lbl_x.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);
    draw_text_ui(LABELS[1], lbl_y.x, lbl_y.y, LABEL_FONT_SIZE, FIELD_TEXT_COLOR);

    clip.offset.x = gui_input_number_f32(module.offset_x_id, inp_x, clip.offset.x);
    clip.offset.y = gui_input_number_f32(module.offset_y_id, inp_y, clip.offset.y);
}

/// Returns every ClipId that has a concrete Clip stored in the map.
fn existing_clip_ids(clips: &HashMap<ClipId, ClipDef>) -> Vec<ClipId> {
    clips.keys().cloned().collect()
}

/// Adds every possible `ClipId` to the supplied Vec.
pub fn fill_all_clip_ids(world_ecs: &WorldEcs, out: &mut Vec<ClipId>) {
    // Built‑in IDs
    let mut ids: Vec<ClipId> = ClipId::iter()
        .filter(|id| !matches!(id, ClipId::New | ClipId::Custom(_)))
        .collect();

    // Gather every custom type
    let mut custom_names = HashSet::new();
    for animation in world_ecs.get_store::<Animation>().data.values() {
        for clip_id in animation.clips.keys() {
            if let ClipId::Custom(name) = clip_id {
                custom_names.insert(name.clone());
            }
        }
    }

    // Sort the custom values
    let mut custom_ids: Vec<ClipId> = custom_names
        .into_iter()
        .map(ClipId::Custom)
        .collect();

    custom_ids.sort_by_key(|id| id.ui_label());

    // Assemble the final list with New at the end
    ids.extend(custom_ids);
    ids.push(ClipId::New);
    *out = ids;
}

/// Helper that moves the currently selected clip under a new `ClipId`.
fn reset_current_clip_id(animation: &mut Animation, new_id: ClipId) {
    let old_id = animation.current.take().unwrap();

    // Take the old clip out of the map
    if let Some(old_clip) = animation.clips.remove(&old_id) {
        // Insert it under the new key
        animation.clips.insert(new_id.clone(), old_clip);
    }

    // Move the runtime state as well
    if let Some(state) = animation.states.remove(&old_id) {
        animation.states.insert(new_id.clone(), state);
    }

    // Finally make the renamed clip the active one
    animation.current = Some(new_id);
}

/// Returns the two label rects and the two input rects for a horizontal pair of fields.
fn layout_pair(
    y: f32,
    rect: Rect,
    labels: [&'static str; 2],
) -> (Rect, Rect, Rect, Rect) {
    // Width of each label
    let width1 = measure_text_ui(labels[0], LABEL_FONT_SIZE, 1.0).width + COLON_GAP;
    let width2 = measure_text_ui(labels[1], LABEL_FONT_SIZE, 1.0).width + COLON_GAP;

    // First label
    let label1 = Rect::new(
        rect.x + PADDING,
        y + LABEL_Y_OFFSET,
        width1,
        INPUT_HEIGHT,
    );

    // First input
    let input0 = Rect::new(label1.x + width1, y, NUM_FIELD_W, INPUT_HEIGHT);

    // Second label
    let label2 = Rect::new(
        input0.x + NUM_FIELD_W + FIELD_GAP,
        y + LABEL_Y_OFFSET,
        width2,
        INPUT_HEIGHT,
    );
    
    // Second input
    let input1 = Rect::new(label2.x + width2, y, NUM_FIELD_W, INPUT_HEIGHT);

    (label1, input0, label2, input1)
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