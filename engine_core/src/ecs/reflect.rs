// engine_core/src/ecs/reflect.rs

/// One mutable field value.
pub enum FieldValue<'a> {
    Text(&'a mut String),
    Float(&'a mut f32),
    Bool(&'a mut bool),
}

/// Metadata that the inspector consumes.
pub struct FieldInfo<'a> {
    pub name: &'static str,
    pub value: FieldValue<'a>,
}

/// Trait that every component needs to expose.
pub trait Reflect {
    /// Returns a vector of mutable descriptors for all fields.
    fn fields(&mut self) -> Vec<FieldInfo<'_>>;
}

/// Helper trait
pub trait ReflectField {
    fn field_info<'a>(field: &'a mut Self, name: &'static str) -> FieldInfo<'a>;
}

impl ReflectField for String {
    fn field_info<'a>(field: &'a mut Self, name: &'static str) -> FieldInfo<'a> {
        FieldInfo { name, value: FieldValue::Text(field) }
    }
}

impl ReflectField for f32 {
    fn field_info<'a>(field: &'a mut Self, name: &'static str) -> FieldInfo<'a> {
        FieldInfo { name, value: FieldValue::Float(field) }
    }
}

impl ReflectField for bool {
    fn field_info<'a>(field: &'a mut Self, name: &'static str) -> FieldInfo<'a> {
        FieldInfo { name, value: FieldValue::Bool(field) }
    }
}