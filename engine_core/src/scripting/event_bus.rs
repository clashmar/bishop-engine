// engine_core/src/scripting/event_bus.rs
use crate::*;
use std::collections::HashMap;
use std::sync::Mutex;
use mlua::Variadic;
use std::sync::Arc;
use mlua::Function;
use mlua::Value;

/// Event dispatcher that stores Lua callbacks.
#[derive(Default, Clone)]
pub struct EventBus {
    /// Maps an event name to list of Lua functions.
    listeners: Arc<Mutex<HashMap<String, Vec<Function>>>>,
}

impl EventBus {
    /// Register a listener for `event`. The `Function` is a Lua closure.
    pub fn on(&self, event: String, handler: Function) {
        let mut map = self.listeners.lock().unwrap();
        map.entry(event).or_default().push(handler);
    }

    /// Emit an event. All registered handlers are called with the supplied arguments.
    pub fn emit(&self, event: String, args: Variadic<Value>) {
        let map = self.listeners.lock().unwrap();
        if let Some(callbacks) = map.get(&event) {
            for cb in callbacks {
                if let Err(e) = cb.call::<()>(args.clone()) {
                    // `onscreen_error!` is the macro used throughout the code base.
                    onscreen_error!("Lua listener error for event '{}': {}", event, e);
                }
            }
        }
    }
}