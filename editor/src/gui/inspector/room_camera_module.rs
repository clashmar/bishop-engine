// editor/src/gui/inspector/camera_module.rs
use engine_core::ecs::entity::Entity;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::ecs::module::InspectorModule;
use engine_core::game::game::GameCtx;
use strum::IntoEnumIterator;
use engine_core::{camera::game_camera::*, ui::text::*};
use engine_core::ecs::module::CollapsibleModule;
use engine_core::ecs::module_factory::ModuleFactoryEntry;
use macroquad::prelude::*;
use engine_core::ui::widgets::*;

pub const ROOM_CAMERA_MODULE_TITLE: &str = "Room Camera";

#[derive(Default)]
pub struct RoomCameraModule {
    pub mode_id: WidgetId,
    pub zoom_id: WidgetId,
    pub slider_id: WidgetId,
    pub cam_mode_id: WidgetId,
}

impl InspectorModule for RoomCameraModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<RoomCamera>(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        game_ctx: &mut GameCtx,
        entity: Entity,
    ) {
        let world_ecs = &mut game_ctx.cur_world_ecs;

        let cam = world_ecs
            .get_mut::<RoomCamera>(entity)
            .expect("Camera must exist");

        let mut y = rect.y + WIDGET_SPACING;

        // Layout dropdown now but draw at the end
        let mode_label = "Zoom Mode: ";
        let label_width = measure_text_ui(mode_label, FIELD_TEXT_SIZE_16, 1.0).width;
        draw_text_ui(mode_label, rect.x, y + 20.0, FIELD_TEXT_SIZE_16, FIELD_TEXT_COLOR);

        let mode_rect = Rect::new(rect.x + label_width + WIDGET_SPACING, y, rect.w - label_width - WIDGET_SPACING, 30.0);
        let current_mode = cam.zoom_mode;
        let current_label = format!("{current_mode}");
        let zoom_options: Vec<ZoomMode> = ZoomMode::iter()
            .collect();

        // Advance y for the next position
        y += mode_rect.h + mode_rect.h + WIDGET_SPACING;

        match cam.zoom_mode   {
            ZoomMode::Step => {
                // Fields to change the global virtual screen dimensions
                let scale_rect = Rect::new(
                    rect.x,
                    y,
                    rect.w,
                    40.0,               
                );

                const STEPS: &[f32; 4] = &[0.5_f32, 1.0, 2.0, 3.0];
                
                let current_scalar = 2.0 / (cam.zoom.x * world_virtual_width());
                let new_scalar = gui_stepper(scale_rect, "Scale", STEPS, current_scalar);

                if (new_scalar - current_scalar).abs() > f32::EPSILON {
                    let width = world_virtual_width() * new_scalar;
                    let height = world_virtual_height() * new_scalar;
                    cam.zoom = vec2(1.0 / width * 2.0, 1.0 / height * 2.0);
                }
            }
            ZoomMode::Free => {
                // Editable numeric field and slider for zoom
                let zoom_rect = Rect::new(
                    rect.x,
                    y,
                    rect.w,
                    35.0,
                );

                self.draw_freeform_mode(
                    zoom_rect,
                    cam,
                );
            }
        }

        // Advance y for the next position
        y += 30.0;

        // Camera mode
        let cam_mode_label = "Camera Mode: ";
        let cam_label_width = measure_text_ui(cam_mode_label, FIELD_TEXT_SIZE_16, 1.0).width;
        draw_text_ui(
            cam_mode_label,
            rect.x,
            y + 20.0,
            FIELD_TEXT_SIZE_16,
            FIELD_TEXT_COLOR,
        );

        let cam_mode_rect = Rect::new(
            rect.x + cam_label_width + WIDGET_SPACING,
            y,
            rect.w - cam_label_width - WIDGET_SPACING,
            30.0,
        );

        // Current value & label
        let current_cam_mode = cam.camera_mode;
        let current_cam_label = format!("{current_cam_mode}");
        let cam_mode_options: Vec<CameraMode> = vec![
            CameraMode::Fixed,
            CameraMode::Follow(FollowRestriction::Free),
            CameraMode::Follow(FollowRestriction::ClampY),
            CameraMode::Follow(FollowRestriction::ClampX),
        ];

        // Advance y for the next position
        // y += cam_mode_rect.h + SPACING;

        // Render the dropdowns in reverse order
        if let Some(new_cam_mode) = gui_dropdown(
            self.cam_mode_id,
            cam_mode_rect,
            &current_cam_label,
            &cam_mode_options,
            |mode| mode.ui_label(),
        ) {
            if new_cam_mode != current_cam_mode {
                cam.camera_mode = new_cam_mode;
            }
        }
        
        if let Some(new_mode) = gui_dropdown(
            self.mode_id, 
            mode_rect, 
            &current_label, 
            &zoom_options,
            |mode| mode.ui_label(),
        ) {
            if new_mode != current_mode {
                cam.zoom_mode = new_mode;
            }
        }
    }
}

impl RoomCameraModule {
    // Draw a single numeric field that edits the scalar zoom.
    fn draw_freeform_mode(
        &self,
        rect: Rect,
        cam: &mut RoomCamera,
    ) {
        let scalar = 2.0 / (cam.zoom.x * world_virtual_width());

        const MIN: f32 = 0.5;
        const MAX: f32 = 3.0;
        let scalar = scalar.clamp(MIN, MAX);

        // Label
        let label = "Scale: ";
        let label_width = measure_text_ui(label, FIELD_TEXT_SIZE_16, 1.0).width + 1.0;
        let num_width = measure_text_ui("0.00", FIELD_TEXT_SIZE_16, 1.0).width;
        draw_text_ui(label, rect.x, rect.y, FIELD_TEXT_SIZE_16, FIELD_TEXT_COLOR);

        // Numeric field 
        let num_rect = Rect::new(
            rect.x + label_width,
            rect.y - FIELD_TEXT_SIZE_16,
            num_width + WIDGET_SPACING,
            rect.h,
        );

        // Slider
        let slider_rect = Rect::new(
            rect.x + label_width + num_width + 2.0 * WIDGET_SPACING,
            rect.y - FIELD_TEXT_SIZE_16,
            rect.w - (label_width + num_width + 2.0 * WIDGET_SPACING),
            rect.h,
        );

        // Numeric field
        let typed = gui_input_number_f32(
            self.zoom_id,
            num_rect,
            round_to_dp(scalar, 2),
        );

        // Slider
        let (slider_val, slider_changed) = gui_slider(
            self.slider_id,
            slider_rect,
            MIN, // min
            MAX, // max
            round_to_dp(scalar, 2),
        );      

        // Resolve the new scalar
        let mut new_scalar = scalar;
        if (typed - scalar).abs() > f32::EPSILON {
            new_scalar = round_to_dp(typed, 2).clamp(MIN, MAX);
        }
        if slider_changed {
            new_scalar = round_to_dp(slider_val, 2).clamp(MIN, MAX);
        }
        
        // Write back if anything changed
        if (new_scalar - scalar).abs() > f32::EPSILON {
            let width = world_virtual_width() * new_scalar;
            let height = world_virtual_height() * new_scalar;
            cam.zoom = vec2(1.0 / width * 2.0, 1.0 / height * 2.0);
        }
    }
}

inventory::submit! {
    ModuleFactoryEntry {
        title: ROOM_CAMERA_MODULE_TITLE,
        factory: || {
            Box::new(
                CollapsibleModule::new(
                    crate::gui::inspector::room_camera_module::RoomCameraModule::default()
                )
                .with_title(ROOM_CAMERA_MODULE_TITLE)
            )
        },
    }
}

/// Return `v` rounded to *exactly* `dp` decimal places and stripped of the
/// floating‑point noise that would otherwise appear as …99999 or …00001.
#[inline]
fn round_to_dp(v: f32, dp: u32) -> f32 {
    let factor = 10_f32.powi(dp as i32);

    let mut r = (v * factor).round() / factor;

    if (r - r.round()).abs() < 1e-6 {
        r = r.round();
    }
    r
}