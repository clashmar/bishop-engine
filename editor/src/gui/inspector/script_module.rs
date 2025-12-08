// editor/src/gui/inspector/script_module.rs
use std::collections::HashMap;
use engine_core::ecs::reflect_field::parse_field_name;
use engine_core::ecs::module_factory::ModuleFactoryEntry;
use engine_core::script::script::ScriptField;
use engine_core::ui::text::*;
use engine_core::game::game::*;
use engine_core::*;
use engine_core::ui::widgets::*;
use engine_core::script::script::Script;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::ecs::module::{CollapsibleModule, InspectorModule};
use macroquad::prelude::*;

#[derive(Default)]   
pub struct ScriptModule {
    field_ids: HashMap<String, WidgetId>,
    fields_len: usize,
}

const TOP_PADDING: f32 = 10.0;
const SPACING: f32 = 5.0;
const FONT_SIZE: f32 = DEFAULT_FONT_SIZE_16;
const MIN_LABEL_WIDTH: f32 = 80.0;
const MIN_WIDGET_WIDTH: f32 = 80.0;
const LABEL_PADDING: f32 = 10.0;

impl InspectorModule for ScriptModule {
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        world_ecs.get::<Script>(entity).is_some()
    }

    fn removable(&self) -> bool { true }

    fn remove(&mut self, world_ecs: &mut WorldEcs, entity: Entity) {
        world_ecs.get_store_mut::<Script>().remove(entity);
    }

    fn draw(
        &mut self,
        rect: Rect,
        game_ctx: &mut GameCtx,
        entity: Entity,
    ) {
        let world_ecs = &mut game_ctx.cur_world_ecs;
        let script_manager = &mut game_ctx.script_manager;

        let script_comp = if let Some(comp) = world_ecs.get_mut::<Script>(entity) {
            comp
        } else {
            return;
        };

        if script_comp.table.is_none() && script_comp.script_id.0 != 0 {
            // TODO: script manager should load the script?
            if let Err(e) = script_comp.load(script_manager) {
                onscreen_error!("Failed to load script: {}", e);
            }
        }

        // Layout
        let mut y = rect.y + WIDGET_SPACING;          
        let full_w = rect.w - 2.0 * WIDGET_PADDING;                

        // Picker
        let button_size = DEFAULT_FIELD_HEIGHT;

        let picker_rect = Rect::new(
            rect.x + WIDGET_PADDING,
            y,
            full_w - button_size - SPACING,
            DEFAULT_FIELD_HEIGHT,
        );

        let refresh_rect = Rect::new(
            picker_rect.x + picker_rect.w + SPACING,
            y,
            button_size,
            button_size,
        );

        if gui_script_picker(picker_rect, &mut script_comp.script_id, script_manager) {
            script_comp.table = None; // force re‑load on next frame
            script_comp.data.fields.clear();
        }

        if gui_button(refresh_rect, "R") {
            // TODO: Make button with picture 
            // Force full script reload on next frame
            script_comp.table = None;
        }

        y += picker_rect.h + SPACING * 2.0;

        // Draw each field if loaded
        if script_comp.table.is_some() {
            // Include picker in field count
            self.fields_len = script_comp.data.fields.len() + 1;

            let mut field_names: Vec<_> = script_comp.data.fields
                .keys()
                .cloned()
                .collect();

            // Ensures deterministic order
            field_names.sort();

            for name in field_names {
                // Create the id for the widget
                let base_key = name.to_string();
                let base_id = *self
                    .field_ids
                    .entry(base_key.clone())
                    .or_insert_with(WidgetId::default);

                // Prepare the field label
                let display_name = parse_field_name(&name);
                let label = format!("{} :", display_name);
                let label_w = measure_text_ui(&label, FONT_SIZE, 1.0).width.max(MIN_LABEL_WIDTH);
                let widget_x = rect.x + label_w + LABEL_PADDING;
                draw_text_ui(&label, rect.x, y + 22.0, FONT_SIZE, FIELD_TEXT_COLOR);

                // Widget rectangle
                let widget_x = if widget_x > rect.x + rect.w - MIN_WIDGET_WIDTH {
                    // Clamp the widget size to the min length
                    rect.x + rect.w - MIN_WIDGET_WIDTH
                } else {
                    widget_x
                };

                let widget_w = (rect.x + rect.w) - widget_x - 10.0;
                let widget_rect = Rect::new(widget_x, y, widget_w.max(MIN_WIDGET_WIDTH), DEFAULT_FIELD_HEIGHT);

                // Pull the mutable reference to the field value
                let field = match script_comp
                    .data
                    .fields
                    .get_mut(&name) {
                        Some(f) => f,
                        None =>  {
                            onscreen_error!("Could not read field data from script component.");
                            return
                        },
                    };

                // Track if any values changed to write back
                let mut changed = false;

                match field {
                    ScriptField::Bool(ref mut v) => {
                        let cb_rect = Rect::new(
                            widget_rect.x,
                            widget_rect.y + 6.0,
                            DEFAULT_CHECKBOX_DIMS,
                            DEFAULT_CHECKBOX_DIMS,
                        );
                        if gui_checkbox(cb_rect, v) {
                            changed = true;
                        }
                    }
                    ScriptField::Int(ref mut v) => {
                        let new = gui_input_number_i32(base_id, widget_rect, *v as i32) as i64;
                        if new != *v {
                            *v = new;
                            changed = true;
                        }
                    }
                    ScriptField::Float(ref mut v) => {
                        let new = gui_input_number_f32(base_id, widget_rect, *v as f32) as f64;
                        if new != *v {
                            *v = new;
                            changed = true;
                        }
                    }
                    ScriptField::Text(ref mut s) => {
                        let (txt, _) = gui_input_text_default(base_id, widget_rect, s);
                        if txt != *s {
                            *s = txt;
                            changed = true;
                        }
                    }
                    ScriptField::Vec2(ref mut v) => {
                        let id_x = *self
                            .field_ids
                            .entry(format!("{}.x", name))
                            .or_insert_with(WidgetId::default);

                        let id_y = *self
                            .field_ids
                            .entry(format!("{}.y", name))
                            .or_insert_with(WidgetId::default);

                        let half = widget_rect.w / 2.0;

                        // X
                        let rect_x = Rect::new(widget_rect.x, widget_rect.y, half - 2.0, widget_rect.h);
                        let new_x = gui_input_number_f32(id_x, rect_x, v[0]);
                        if (new_x - v[0]).abs() > f32::EPSILON {
                            v[0] = new_x;
                            changed = true;
                        }

                        // Y
                        let rect_y = Rect::new(
                            widget_rect.x + half + 2.0,
                            widget_rect.y,
                            half - 2.0,
                            widget_rect.h,
                        );

                        let new_y = gui_input_number_f32(id_y, rect_y, v[0]);
                        if (new_y - v[0]).abs() > f32::EPSILON {
                            v[0] = new_y;
                            changed = true;
                        };
                    }
                    ScriptField::Vec3(ref mut v) => {
                        let id_x = *self
                            .field_ids
                            .entry(format!("{}.x", name))
                            .or_insert_with(WidgetId::default);

                        let id_y = *self
                            .field_ids
                            .entry(format!("{}.y", name))
                            .or_insert_with(WidgetId::default);

                        let id_z = *self
                            .field_ids
                            .entry(format!("{}.z", name))
                            .or_insert_with(WidgetId::default);

                        let third = widget_rect.w / 3.0 - SPACING / 3.0;

                        // X
                        let rect_x = Rect::new(widget_rect.x, widget_rect.y, third - 2.0, widget_rect.h);
                        let new_x = gui_input_number_f32(id_x, rect_x, v[0]);
                        if (new_x - v[0]).abs() > f32::EPSILON {
                            v[0] = new_x;
                            changed = true;
                        }

                        // Y
                        let rect_y = Rect::new(
                            widget_rect.x + third + 2.0,
                            widget_rect.y,
                            third - 2.0,
                            widget_rect.h,
                        );

                        let new_y = gui_input_number_f32(id_y, rect_y, v[0]);
                        if (new_y - v[0]).abs() > f32::EPSILON {
                            v[0] = new_y;
                            changed = true;
                        };

                        // Z
                        let rect_z = Rect::new(
                            widget_rect.x + 2.0 * third + 4.0,
                            widget_rect.y,
                            third - 2.0,
                            widget_rect.h,
                        );

                        let new_z = gui_input_number_f32(id_z, rect_z, v[0]);
                        if (new_z - v[0]).abs() > f32::EPSILON {
                            v[0] = new_z;
                            changed = true;
                        };
                    }
                }

                // Write back to Lua
                if changed {
                    if let Err(e) = script_comp.sync_to_lua(script_manager) {
                        onscreen_error!("Failed to sync script: {}", e);
                    }
                }

                y += widget_rect.h + SPACING;
            }
        }
        else {
            self.fields_len = 1;
        }
    }

    /// Compute the height from the number of fields
    fn height(&self) -> f32 {
        // Total height = top padding + picker gap + (field height + spacing) * count (including picker)
        TOP_PADDING + SPACING + self.fields_len as f32 * (DEFAULT_FIELD_HEIGHT + SPACING)
    }
}

inventory::submit! {
    ModuleFactoryEntry {
        title: <engine_core::script::script::Script>::TYPE_NAME,
        factory: || {
            Box::new(
                CollapsibleModule::new(
                    crate::gui::inspector::script_module::ScriptModule::default()
                )
                .with_title(<engine_core::script::script::Script>::TYPE_NAME)
            )
        },
    }
}