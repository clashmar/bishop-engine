use engine_core::camera::game_camera::{world_virtual_height, world_virtual_width};
// editor/src/gui/inspector/camera_module.rs
use engine_core::ecs::component::RoomCamera;
use engine_core::ecs::module::CollapsibleModule;
use engine_core::ecs::module_factory::ModuleFactoryEntry;
use engine_core::global::{cam_tile_dims, set_global_cam_tile_dims, tile_size};
use engine_core::tiles::tile;
use macroquad::prelude::*;
use engine_core::ui::widgets::*;
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::{
        entity::Entity, 
        module::InspectorModule, 
        world_ecs::WorldEcs
    }
};
use crate::gui::gui_constants::FIELD_TEXT_SIZE;


pub const ROOM_CAMERA_MODULE_TITLE: &str = "Room Camera";
const SPACING: f32 = 5.0;

#[derive(Default)]
pub struct RoomCameraModule {
    pub mode: CameraMode,
    pub x_id: WidgetId,
    pub y_id: WidgetId,
    pub zoom_id: WidgetId,
    pub slider_id: WidgetId,
}

/// The two display modes the inspector can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum CameraMode {
    #[default]
    Grid,
    FreeForm,
}

impl InspectorModule for RoomCameraModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<RoomCamera>(entity).is_some()
    }

    fn draw(
        &mut self,
        rect: Rect,
        _asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        let cam = world_ecs
            .get_mut::<RoomCamera>(entity)
            .expect("Camera must exist");

        match self.mode   {
            CameraMode::Grid => {
                // Fields to change the global virtual screen dimensions
                let dims_rect = Rect::new(
                    rect.x,
                    rect.y + 30.0,
                    rect.w,
                    40.0,
                );

                self.draw_dims_fields(dims_rect);

                let width = world_virtual_width();
                let height = world_virtual_height();

                let zoom = vec2(1. / width * 2., 1. / height * 2.);
                cam.zoom = zoom;
            }
            CameraMode::FreeForm => {
                // Editable numeric field and slider for zoom
                let zoom_rect = Rect::new(
                    rect.x,
                    rect.y + 30.0,
                    rect.w,
                    40.0,
                );

                // self.draw_zoom_field(
                //     zoom_rect,
                //     cam,
                // );
            }
        }
    }
}

impl RoomCameraModule {
    fn draw_dims_fields(
        &self,
        rect: Rect,
    ) {
        // Global Dimensions label
        let zoom_label = "Global Dimensions: ";
        let label_width = measure_text(zoom_label, None, FIELD_TEXT_SIZE as u16, 1.0).width;
        draw_text(zoom_label, rect.x + 2.0, rect.y + 5.0, FIELD_TEXT_SIZE, WHITE);

        let num_text = "000"; // So width will never bigger than a three digit number
        let num_width = measure_text(&num_text, None, FIELD_TEXT_SIZE as u16, 1.0).width;

        // Virtual width x
        let x_rect = Rect::new(
            rect.x + label_width,
            rect.y - FIELD_TEXT_SIZE,
            num_width + SPACING,
            rect.h,
        );

        // Virtual width y
        let y_rect = Rect::new(
            x_rect.x + x_rect.w + SPACING,
            rect.y - FIELD_TEXT_SIZE,
            num_width + SPACING,
            rect.h,
        );

        let (x, y) = cam_tile_dims();

        let new_x = gui_input_number_i32(self.x_id, x_rect, x as i32);
        if new_x != x as i32 {
            set_global_cam_tile_dims((new_x as f32, y));
        }

        let new_y = gui_input_number_i32(self.y_id, y_rect, y as i32);
        if new_y != y as i32 {
            set_global_cam_tile_dims((x, new_y as f32));
        }
    }


    // / Draw a single numeric field that edits the scalar zoom.
    // fn draw_zoom_field(
    //     &self,
    //     rect: Rect,
    //     cam: &mut RoomCamera,
    // ) {
    //     // To keep zoom values nice to work with
    //     const DISPLAY_FACTOR: f32 = 1000.0;
    //     const DECIMAL_PLACES: u32 = 5;

    //     // Current scalar
    //     let scalar = cam.scalar_zoom; 
    //     let rounded_scalar = round_to_dp(scalar, DECIMAL_PLACES);

    //     // Zoom label
    //     let zoom_label = "Zoom: ";
    //     let label_width = measure_text(zoom_label, None, FIELD_TEXT_SIZE as u16, 1.0).width;
    //     draw_text(zoom_label, rect.x + 2.0, rect.y + 5.0, FIELD_TEXT_SIZE, WHITE);

    //     let num_text = "000000"; // So width will never bigger than a five digit number
    //     let num_width = measure_text(&num_text, None, FIELD_TEXT_SIZE as u16, 1.0).width;

    //     // Zoom Numeric field 
    //     let num_rect = Rect::new(
    //         rect.x + label_width,
    //         rect.y - FIELD_TEXT_SIZE,
    //         num_width + SPACING,
    //         rect.h,
    //     );

    //     // Slider
    //     let slider_rect = Rect::new(
    //         rect.x + label_width + num_width + 2.0 * SPACING,
    //         rect.y - FIELD_TEXT_SIZE,
    //         rect.w - (label_width + num_width + 3.0 * SPACING),
    //         rect.h,
    //     );

    //     let typed = gui_input_number_f32(
    //         self.zoom_id,
    //         num_rect, 
    //         round_to_dp(rounded_scalar * DISPLAY_FACTOR, DECIMAL_PLACES)
    //     );

    //     let mut new_scalar = scalar; 

    //     if (typed - rounded_scalar).abs() > f32::EPSILON {
    //         // User typed a new number
    //         new_scalar = round_to_dp(typed / DISPLAY_FACTOR, DECIMAL_PLACES);
    //     }

    //     // Slider
    //     let (slider_val, slider_changed) = gui_slider(
    //         self.slider_id,
    //         slider_rect,
    //         0.001, // min
    //         0.05, // max       
    //         rounded_scalar,
    //     );        

    //     if slider_changed {
    //         new_scalar = slider_val;
    //     }
        
    //     // Write back if anything changed
    //     if (new_scalar - scalar).abs() > f32::EPSILON {
    //         cam.scalar_zoom = new_scalar;
    //     }
    // }
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