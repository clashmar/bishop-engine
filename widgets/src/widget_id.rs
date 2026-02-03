use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WidgetId(pub usize);

impl Default for WidgetId {
    fn default() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        WidgetId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
