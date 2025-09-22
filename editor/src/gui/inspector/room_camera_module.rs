// editor/src/gui/inspector/camera_module.rs
use engine_core::ecs::component::RoomCamera;
use engine_core::ecs::module::CollapsibleModule;
use engine_core::ecs::module_factory::ModuleFactoryEntry;
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

#[derive(Default)]
pub struct RoomCameraModule {}

impl RoomCameraModule {
    /// Draw a **single** numeric field that edits the *scalar* zoom.
    /// The scalar is converted to a non‑uniform `Vec2` that respects the
    /// current screen aspect.
    fn draw_zoom_field(
        &self,
        rect: Rect,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        let cam = world_ecs
            .get_mut::<RoomCamera>(entity)
            .expect("Camera must exist");

        // To keep zoom values nice to work with
        const DISPLAY_FACTOR: f32 = 1000.0;
        const DECIMAL_PLACES: u32 = 5;
        const SPACING: f32 = 5.0;

        // Current scalar
        let scalar = cam.scalar_zoom; 
        let rounded_scalar = round_to_dp(scalar, DECIMAL_PLACES);

        // Zoom label
        let zoom_label = "Zoom: ";
        let font_size_zoom = 20.0;
        let label_width = measure_text(zoom_label, None, font_size_zoom as u16, 1.0).width;
        draw_text(zoom_label, rect.x + 2.0, rect.y + 5.0, font_size_zoom, WHITE);

        let num_text = "000000"; // So width will never bigger than a five digit number
        let num_width = measure_text(&num_text, None, font_size_zoom as u16, 1.0).width;

        // Numeric field 
        let num_rect = Rect::new(
            rect.x + label_width,
            rect.y - font_size_zoom,
            num_width + SPACING,
            rect.h,
        );

        // Slider
        let slider_rect = Rect::new(
            rect.x + label_width + num_width + 2.0 * SPACING,
            rect.y - font_size_zoom,
            rect.w - (label_width + num_width + 3.0 * SPACING),
            rect.h,
        );

        let typed = gui_input_number(
            num_rect, 
            round_to_dp(rounded_scalar * DISPLAY_FACTOR, DECIMAL_PLACES)
        );

        let mut new_scalar = scalar; 

        if (typed - rounded_scalar).abs() > f32::EPSILON {
            // User typed a new number
            new_scalar = round_to_dp(typed / DISPLAY_FACTOR, DECIMAL_PLACES);
        }

        // Slider
        let (slider_val, slider_changed) = gui_slider(
            slider_rect,
            0.001, // min
            0.05, // max       
            rounded_scalar,
        );        

        if slider_changed {
            new_scalar = slider_val;
        }
        
        // Write back if anything changed
        if (new_scalar - scalar).abs() > f32::EPSILON {
            cam.scalar_zoom = new_scalar;
        }
    }
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
        // Editable numeric field for zoom
        let edit_rect = Rect::new(
            rect.x,
            rect.y + 30.0,
            rect.w,
            40.0,
        );
        self.draw_zoom_field(edit_rect, world_ecs, entity);
    }
}

inventory::submit! {
    ModuleFactoryEntry {
        title: <engine_core::ecs::component::RoomCamera>::TYPE_NAME,
        factory: || {
            Box::new(
                CollapsibleModule::new(
                    crate::gui::inspector::room_camera_module::RoomCameraModule::default()
                )
                .with_title("Camera")
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