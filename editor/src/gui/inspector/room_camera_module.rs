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

        // -------------------------------------------------
        // 1️⃣  Current scalar (the value we store internally)
        // -------------------------------------------------
        let scalar = cam.scalar_zoom;                 // e.g. 0.01, 0.02, …
        const DISPLAY_FACTOR: f32 = 0.01;            // keep the old “1 = 0.01” mapping
        let display_val = scalar / DISPLAY_FACTOR;    // show as 1, 2, 3 …

        // -------------------------------------------------
        // 2️⃣  Layout – label | numeric field | slider
        // -------------------------------------------------
        let label_w = 40.0;
        let num_w   = 60.0;
        let spacing = 5.0;
        let slider_rect = Rect::new(
            rect.x + label_w + num_w + 2.0 * spacing,
            rect.y,
            rect.w - (label_w + num_w + 3.0 * spacing),
            rect.h,
        );

        // -------------------------------------------------
        // 3️⃣  Numeric field (keeps exact typing)
        // -------------------------------------------------
        draw_text("Zoom:", rect.x + 2.0, rect.y + 22.0, 18.0, WHITE);
        let num_rect = Rect::new(
            rect.x + label_w,
            rect.y,
            num_w,
            rect.h,
        );
        let typed = gui_input_number(num_rect, display_val);
        let mut new_scalar = scalar; // fallback – unchanged

        if (typed - display_val).abs() > f32::EPSILON {
            // User typed a new number → convert back to internal scalar
            new_scalar = typed * DISPLAY_FACTOR;
        }

        // -------------------------------------------------
        // 4️⃣  Slider (range 0.5 … 5.0 in *display* units)
        // -------------------------------------------------
        // The slider works on the same “display” unit (1 = 0.01 scalar)
        let (slider_val, slider_changed) = gui_slider(
            slider_rect,
            0.5,                // min zoom (½ × tile width)
            5.0,                // max zoom (5 × tile width)
            display_val,
        );
        if slider_changed {
            new_scalar = slider_val * DISPLAY_FACTOR;
        }

        // -------------------------------------------------
        // 5️⃣  Write back if anything changed
        // -------------------------------------------------
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