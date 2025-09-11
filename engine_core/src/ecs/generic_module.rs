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
}};
use macroquad::prelude::*;
use std::{borrow::Cow, marker::PhantomData};

/// A thin wrapper that can draw *any* `T: Reflect`.
pub struct GenericModule<T> {
    _phantom: PhantomData<T>,
}

impl<T> Default for GenericModule<T> {
    fn default() -> Self {
        Self { 
            _phantom: PhantomData 
        }
    }
}

impl<T> InspectorModule for GenericModule<T>
where
    T: Reflect + Component + 'static,
{
    fn visible(&self, world_ecs: &WorldEcs, entity: Entity) -> bool {
        // Use the new `get_store` helper.
        world_ecs.get_store::<T>().contains(entity)
    }

    fn draw(
        &mut self,
        rect: Rect,
        _assets: &mut AssetManager,
        world_ecs: &mut WorldEcs,
        entity: Entity,
    ) {
        // Grab a mutable reference to the component instance.
        let component = world_ecs
            .get_store_mut::<T>()
            .get_mut(entity)
            .expect("Component must exist for selected entity");

        // Layout constants
        let mut y = rect.y + 10.0;
        let label_w = 80.0;
        let field_h = 30.0;
        let spacing = 5.0;

        // Iterate over the fields supplied by the `Reflect` impl.
        for field in component.fields() {

            // Draw the field label.
            let label = capitalise(field.name);
            draw_text(&label, rect.x, y + 22.0, 18.0, WHITE);

            // Widget rectangle.
            let widget_rect = Rect::new(rect.x + label_w, y, rect.w - label_w - 10.0, field_h);

            // Dispatch based on the enum variant.
            match field.value {
                FieldValue::Text(txt) => {
                    // `txt` is `&mut String`.
                    let new = gui_input_text(widget_rect, txt.as_str());
                    if new != *txt {
                        *txt = new;
                    }
                }
                FieldValue::Float(num) => {
                    // `num` is `&mut f32`.
                    let new = gui_input_number(widget_rect, *num);
                    if (new - *num).abs() > f32::EPSILON {
                        *num = new;
                    }
                }
                FieldValue::Bool(b) => {
                    // `b` is `&mut bool`.
                    let mut v = *b;
                    if gui_checkbox(widget_rect, &mut v) {
                        *b = v;
                    }
                }
            }

            y += field_h + spacing;
        }
    }

    fn height(&self) -> f32 {
        // Rough estimate – the inspector will call `draw` each frame,
        // so returning a generous constant is fine.
        200.0
    }
}

fn capitalise(name: &str) -> Cow<str> {
    // Fast path – already starts with an ASCII upper‑case letter.
    if name
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
    {
        return Cow::Borrowed(name);
    }

    // Build a new owned string with the first char upper‑cased.
    let mut chars = name.chars();
    let first = chars.next().map(|c| c.to_ascii_uppercase());
    let rest: String = chars.collect();
    match first {
        Some(f) => Cow::Owned(format!("{}{}", f, rest)),
        None => Cow::Borrowed(name), // empty string – should never happen
    }
}