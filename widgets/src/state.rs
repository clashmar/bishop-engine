use std::cell::RefCell;
use std::collections::HashMap;
use crate::WidgetId;

thread_local! {
    pub static INPUT_TEXT_STATE: RefCell<HashMap<WidgetId, (String, usize, bool, f64, bool)>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    pub static INPUT_NUMBER_STATE: RefCell<HashMap<WidgetId, (String, usize, bool)>> =
        RefCell::new(HashMap::new());
}

thread_local! {
    pub static INPUT_FOCUSED: RefCell<bool> = RefCell::new(false);
}

pub fn input_is_focused() -> bool {
    INPUT_FOCUSED.with(|f| {
        let flag = f.borrow_mut();
        *flag
    })
}

thread_local! {
    pub static DROPDOWN_OPEN: RefCell<bool> = RefCell::new(false);
}

pub fn is_dropdown_open() -> bool {
    DROPDOWN_OPEN.with(|f| *f.borrow())
}

pub fn clear_all_input_focus() {
    INPUT_FOCUSED.with(|f| *f.borrow_mut() = false);
    INPUT_TEXT_STATE.with(|s| {
        let mut map = s.borrow_mut();
        for (_, entry) in map.iter_mut() {
            entry.2 = false;
        }
    });
    INPUT_NUMBER_STATE.with(|s| {
        let mut map = s.borrow_mut();
        for (_, entry) in map.iter_mut() {
            entry.2 = false;
        }
    });
}
