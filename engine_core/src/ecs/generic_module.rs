// engine_core/src/ecs/generic_module.rs
use crate::ecs::module::InspectorModule;
use crate::ui::widgets::*;
use crate::{
    assets::asset_manager::AssetManager, 
    ecs::{
        component::Component,
        entity::Entity, 
        reflect::{FieldValue, Reflect}, 
        world_ecs::WorldEcs
    }
};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::{borrow::Cow, marker::PhantomData};

const TOP_PADDING: f32 = 10.0;
const FIELD_HEIGHT: f32 = 30.0;
const SPACING: f32 = 5.0;

/// A thin wrapper that can draw *any* `T: Reflect`.
pub struct GenericModule<T> {
    _phantom: PhantomData<T>,
    field_ids: HashMap<String, WidgetId>,
}

impl<T> Default for GenericModule<T> {
    fn default() -> Self {
        Self { 
            _phantom: PhantomData,
            field_ids: HashMap::new(),
        }
    }
}

impl<T> InspectorModule for GenericModule<T>
where
    T: Reflect + Component + Default + 'static,
{
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        // Use the new `get_store` helper
        world_ecs.get_store::<T>().contains(entity)
    }

    fn draw(
        &mut self,
        rect: Rect,
        asset_manager: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        // Grab a mutable reference to the component instance
        let component = world_ecs
            .get_store_mut::<T>()
            .get_mut(entity)
            .expect("Component must exist for selected entity");

        // Layout constants
        let mut y = rect.y + TOP_PADDING;
        let label_w = 80.0;

        // Iterate over the fields supplied by the `Reflect` impl
        for field in component.fields() {
            // Create the id for the widget
            let base_key = field.name.to_string();
            let base_id = *self
                .field_ids
                .entry(base_key.clone())
                .or_insert_with(WidgetId::default);

            // Draw the field label
            let label = capitalise(field.name);
            draw_text(&label, rect.x, y + 22.0, 18.0, WHITE);

            // Widget rectangle
            let widget_rect = Rect::new(rect.x + label_w, y, rect.w - label_w - 10.0, FIELD_HEIGHT);
            // Dispatch based on the enum variant
            match (field.value, field.widget_hint) {
                (FieldValue::Text(txt), Some("png")) => {
                    let btn_label = if txt.is_empty() {
                        "[Pick File]".to_string()
                    } else {
                        "[Change File]".to_string()
                    };

                    if gui_button(widget_rect, &btn_label) {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PNG images", &["png"])
                                .pick_file()
                            {
                                *txt = path.to_string_lossy().into_owned();

                                // Load the texure in the asset manager
                                asset_manager.get_or_load(&txt);
                            }
                        }
                    }
                }
                (FieldValue::Text(txt), _) => {
                    let (new, _) = gui_input_text_default(base_id, widget_rect, txt.as_str());
                    if new != *txt {
                        *txt = new;
                    }
                }
                (FieldValue::Float(f), _) => {
                    let new = gui_input_number_f32(base_id, widget_rect, *f);
                    if (new - *f).abs() > f32::EPSILON {
                        *f = new;
                    }
                }
                (FieldValue::Int(i), _) => {
                    let new = gui_input_number_i32(base_id, widget_rect, *i);
                    if new != *i {
                        *i = new;
                    }
                }
                (FieldValue::Bool(b), _) => {
                    let mut v = *b;
                    if gui_checkbox(widget_rect, &mut v) {
                        *b = v;
                    }
                }
                (FieldValue::Vec2(v), _) => {
                    let id_x = *self
                        .field_ids
                        .entry(format!("{}.x", field.name))
                        .or_insert_with(WidgetId::default);

                    let id_y = *self
                        .field_ids
                        .entry(format!("{}.y", field.name))
                        .or_insert_with(WidgetId::default);

                    let half = widget_rect.w / 2.0;

                    // X
                    let rect_x = Rect::new(widget_rect.x, widget_rect.y, half - 2.0, widget_rect.h);
                    let new_x = gui_input_number_f32(id_x, rect_x, v.x);
                    if (new_x - v.x).abs() > f32::EPSILON {
                        v.x = new_x;
                    }
                    // Y
                    let rect_y = Rect::new(
                        widget_rect.x + half + 2.0,
                        widget_rect.y,
                        half - 2.0,
                        widget_rect.h,
                    );
                    let new_y = gui_input_number_f32(id_y, rect_y, v.y);
                    if (new_y - v.y).abs() > f32::EPSILON {
                        v.y = new_y;
                    }
                }
                (FieldValue::Vec3(v), _) => {
                    let id_x = *self
                        .field_ids
                        .entry(format!("{}.x", field.name))
                        .or_insert_with(WidgetId::default);
                    let id_y = *self
                        .field_ids
                        .entry(format!("{}.y", field.name))
                        .or_insert_with(WidgetId::default);
                    let id_z = *self
                        .field_ids
                        .entry(format!("{}.z", field.name))
                        .or_insert_with(WidgetId::default);

                    let third = widget_rect.w / 3.0 - SPACING / 3.0;

                    // X
                    let rect_x = Rect::new(widget_rect.x, widget_rect.y, third, widget_rect.h);
                    let new_x = gui_input_number_f32(id_x, rect_x, v.x);
                    if (new_x - v.x).abs() > f32::EPSILON {
                        v.x = new_x;
                    }
                    // Y
                    let rect_y = Rect::new(
                        widget_rect.x + third + 2.0,
                        widget_rect.y,
                        third,
                        widget_rect.h,
                    );
                    let new_y = gui_input_number_f32(id_y, rect_y, v.y);
                    if (new_y - v.y).abs() > f32::EPSILON {
                        v.y = new_y;
                    }
                    // Z
                    let rect_z = Rect::new(
                        widget_rect.x + 2.0 * third + 4.0,
                        widget_rect.y,
                        third,
                        widget_rect.h,
                    );
                    let new_z = gui_input_number_f32(id_z, rect_z, v.z);
                    if (new_z - v.z).abs() > f32::EPSILON {
                        v.z = new_z;
                    }
                }
            }

            y += FIELD_HEIGHT + SPACING;
        }
    }

    /// Compute the height from the number of reflected fields
    fn height(&self) -> f32 {
        // Create a temporary default instance of `T` only to query its fields
        let mut temp = T::default();
        let field_count = temp.fields().len() as f32;

        // Total height = top padding + (field height + spacing) * count
        TOP_PADDING + field_count * (FIELD_HEIGHT + SPACING)
    }

    fn removable(&self) -> bool { true }

    fn remove(&mut self, world_ecs: &mut WorldEcs, entity: Entity) {
        world_ecs.get_store_mut::<T>().remove(entity);
    }
}

fn capitalise(name: &str) -> Cow<str> {
    // Fast path – already starts with an ASCII upper‑case letter
    if name
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
    {
        return Cow::Borrowed(name);
    }

    // Build a new owned string with the first char upper‑cased
    let mut chars = name.chars();
    let first = chars.next().map(|c| c.to_ascii_uppercase());
    let rest: String = chars.collect();
    match first {
        Some(f) => Cow::Owned(format!("{}{}", f, rest)),
        None => Cow::Borrowed(name), // empty string – should never happen
    }
}