// engine_core/src/ecs/generic_module.rs
use crate::ecs::module::InspectorModule;
use crate::ui::text::*;
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
const LABEL_PADDING: f32 = 10.0;
const MIN_WIDGET_WIDTH: f32 = 80.0;
const MIN_LABEL_WIDTH: f32 = 80.0;
const FONT_SIZE: f32 = DEFAULT_FONT_SIZE;

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

        // Iterate over the fields supplied by the `Reflect` impl
        for field in component.fields() {
            // Create the id for the widget
            let base_key = field.name.to_string();
            let base_id = *self
                .field_ids
                .entry(base_key.clone())
                .or_insert_with(WidgetId::default);

            // Prepare the field label
            let label = parse_field_name(field.name);
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
            let widget_rect = Rect::new(widget_x, y, widget_w.max(MIN_WIDGET_WIDTH), FIELD_HEIGHT);

            // Dispatch based on the enum variant
            match (field.value, field.widget_hint) {
                (FieldValue::SpriteId(id), _) => {
                    gui_sprite_picker(widget_rect, id, asset_manager);
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

pub fn parse_field_name(name: &str) -> Cow<str> {
    // Fast path
    if !name.contains('_')
        && name
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
    {
        return Cow::Borrowed(name);
    }

    // Split on '_' and capitalise each segment
    let mut parts = name.split('_').filter(|s| !s.is_empty());

    // Build the first part (to avoid an extra allocation when possible)
    let first = match parts.next() {
        Some(p) => {
            let mut chars = p.chars();
            let first_char = chars.next().map(|c| c.to_ascii_uppercase());
            let rest: String = chars.collect();
            match first_char {
                Some(f) => format!("{}{}", f, rest),
                None => String::new(),
            }
        }
        None => return Cow::Borrowed(name), // empty input
    };

    // Append the remaining parts, each preceded by a space
    let result = parts.fold(first, |mut acc, part| {
        let mut chars = part.chars();
        let first_char = chars.next().map(|c| c.to_ascii_uppercase());
        let rest: String = chars.collect();
        match first_char {
            Some(f) => {
                acc.push(' ');
                acc.push_str(&format!("{}{}", f, rest));
            }
            None => {}
        }
        acc
    });

    Cow::Owned(result)
}