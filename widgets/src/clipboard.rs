use arboard::Clipboard;
use std::cell::RefCell;

thread_local! {
    static INTERNAL_CLIPBOARD: RefCell<String> = RefCell::new(String::new());
}

/// Gets text from the system clipboard, falling back to internal buffer.
pub fn clipboard_get_text() -> Option<String> {
    if let Ok(mut clipboard) = Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                return Some(text);
            }
        }
    }

    let internal = INTERNAL_CLIPBOARD.with(|cb| cb.borrow().clone());
    if !internal.is_empty() {
        Some(internal)
    } else {
        None
    }
}

/// Sets text to the system clipboard and internal buffer.
pub fn clipboard_set_text(text: &str) -> bool {
    INTERNAL_CLIPBOARD.with(|cb| *cb.borrow_mut() = text.to_string());

    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
    }

    true
}
