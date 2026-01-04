// engine_core/src/ecs/module_factory.rs
use crate::ecs::generic_module::GenericModule;
use crate::ecs::reflect_field::Reflect;
use crate::ecs::component::Component;
use crate::ecs::inpsector_module::*;
use once_cell::sync::Lazy;

/// Human‑readable names of all components that have been registered with `inspector_module!`.
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
pub fn make_module<T>(title: &str, removable: bool) -> Box<dyn InspectorModule>
where
    T: Component + Reflect + Default + 'static,
{
    // The generic UI we already have.
    Box::new(
        CollapsibleModule::new(GenericModule::<T>::new(removable))
            .with_title(title), 
    )
}

/// Public macro for each component that appears in the inspector.
#[macro_export]
macro_rules! inspector_module {
    // Default case: removable = true
    ($ty:ty) => {
        inspector_module!($ty, removable = true);
    };
    
    // Explicit removable flag
    ($ty:ty, removable = $removable:expr) => {
        // Register a factory entry for `$ty`.
        inventory::submit! {
            crate::ecs::module_factory::ModuleFactoryEntry {
                title: <$ty>::TYPE_NAME,
                factory: || crate::ecs::module_factory::make_module::<$ty>(<$ty>::TYPE_NAME, $removable),
            }
        }
    };
}