// engine_core/src/ecs/module_factory.rs
use crate::ecs::generic_module::GenericModule;
use crate::ecs::reflect_field::Reflect;
use crate::ecs::component::Component;
use crate::ecs::inspector_module::*;
use once_cell::sync::Lazy;

/// Human‑readable names of all components that have been registered with `inspector_module!`.
pub static MODULES: Lazy<Vec<&'static ModuleFactoryEntry>> = Lazy::new(|| {
    inventory::iter::<ModuleFactoryEntry>.into_iter().collect()
});

pub trait InspectorModuleFactory {
    /// Human‑readable name that will be shown as the collapsible title.
    fn title(&self) -> &'static str;
    /// Builds the concrete module.
    fn make(&self) -> Box<dyn InspectorModule>;
}

pub struct ModuleFactoryEntry {
    pub title: &'static str,
    /// Factory that builds the concrete UI module.
    pub factory: fn() -> Box<dyn InspectorModule>,
}

inventory::collect!(ModuleFactoryEntry);

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
    
    ($ty:ty, removable = $removable:expr) => {
        inventory::submit! {
            $crate::ecs::module_factory::ModuleFactoryEntry {
                title: <$ty>::TYPE_NAME,
                factory: || crate::ecs::module_factory::make_module::<$ty>(<$ty>::TYPE_NAME, $removable),
            }
        }
    };
}