// engine_core/src/ecs/module_factory.rs
use once_cell::sync::Lazy;
use crate::ecs::{
    generic_module::GenericModule,
    module::{CollapsibleModule, InspectorModule},
};
use crate::ecs::component::Component;
use crate::ecs::reflect_field::Reflect;

/// Human‑readable names of all components that have been registered with `ecs_component!`.
pub static MODULES: Lazy<Vec<&'static ModuleFactoryEntry>> = Lazy::new(|| {
    inventory::iter::<ModuleFactoryEntry>.into_iter().collect()
});

/// The only thing we need from a factory: give us a ready‑to‑use
/// `Box<dyn InspectorModule>`.
pub trait InspectorModuleFactory {
    /// Human‑readable name that will be shown as the collapsible title.
    fn title(&self) -> &'static str;

    /// Build the concrete module (`CollapsibleModule<GenericModule<T>>`).
    fn make(&self) -> Box<dyn InspectorModule>;
}

/// One entry per component type.
pub struct ModuleFactoryEntry {
    pub title: &'static str,
    /// Factory that builds the concrete UI module.
    pub factory: fn() -> Box<dyn InspectorModule>,
}

// Tell `inventory` to keep a list of those entries.
inventory::collect!(ModuleFactoryEntry);

/// Helper that builds the concrete `Box<dyn InspectorModule>` for a given `T`.
pub fn make_module<T>(title: &str) -> Box<dyn InspectorModule>
where
    T: Component + Reflect + Default + 'static,
{
    // The generic UI we already have.
    Box::new(
        CollapsibleModule::new(GenericModule::<T>::default())
            .with_title(title), 
    )
}

/// Public macro – put it *once* for each component you want to appear
/// in the inspector.  It can live in the same file that defines the
/// component, or in a user‑crate that adds new components.
#[macro_export]
macro_rules! inspector_module {
    ($ty:ty) => {
        // Register a factory entry for `$ty`.
        inventory::submit! {
            crate::ecs::module_factory::ModuleFactoryEntry {
                title: <$ty>::TYPE_NAME,
                factory: || crate::ecs::module_factory::make_module::<$ty>(<$ty>::TYPE_NAME),
            }
        }
    };
}