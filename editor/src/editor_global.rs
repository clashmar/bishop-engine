// editor/src/editor_global.rs
use crate::commands::editor_command_manager::EditorCommandManager;
use crate::commands::editor_command_manager::EditorCommand;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::cell::Cell;
use crate::Editor;
use std::cell::RefCell;

/// Global services that can be reached from anywhere in the editor.
pub struct EditorServices {
    pub editor: RefCell<Option<Editor>>, // set once at startup
    pub command_manager: RefCell<EditorCommandManager>,
    pub pending_undo: Cell<bool>,
    pub pending_redo: Cell<bool>,
    pub entity_clipboard: RefCell<Option<Vec<(String, String)>>>,
}

impl EditorServices {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            editor: RefCell::new(None),
            command_manager: RefCell::new(EditorCommandManager::new()),
            pending_undo: Cell::new(false),
            pending_redo: Cell::new(false),
            entity_clipboard: RefCell::new(None), 
        })
    }
}

thread_local! {
    /// Single instance of services used by the whole program.
    pub static EDITOR_SERVICES: Rc<EditorServices> = EditorServices::new();
}

/// Store the `Editor` in global services.
pub fn set_editor(editor: Editor) {
    EDITOR_SERVICES.with(|services| {
        *services.editor.borrow_mut() = Some(editor);
    });
}

/// Gets mutable access to the `Editor`.
pub fn with_editor<F, R>(f: F) -> R
where
    F: FnOnce(&mut Editor) -> R,
{
    EDITOR_SERVICES.with(|services| {
        let mut opt = services.editor.borrow_mut();
        let editor = opt
            .as_mut()
            .expect("Editor not initialised");
        f(editor)
    })
}

/// Gets async mutable access to the `Editor`.
pub async fn with_editor_async<R, F>(f: F) -> R
where
    for<'a> F: FnOnce(&'a mut Editor) -> Pin<Box<dyn Future<Output = R> + 'a>>,
{
    let services = EDITOR_SERVICES.with(|s| s.clone());

    let mut opt = services.editor.borrow_mut();
    let editor = opt
        .as_mut()
        .expect("Editor not initialised");

    let fut = f(editor);
    fut.await
}

/// Push an `EditorCommand` to the global command queue.
pub fn push_command(command: Box<dyn EditorCommand>) {
    EDITOR_SERVICES.with(|services| {
        services.command_manager.borrow_mut().push(command);
    });
}

/// Apply all `EditorCommand`'s in the global command queue.
pub fn apply_pending_commands() {
    EDITOR_SERVICES.with(|s| {
        // Execute normal commands
        s.command_manager.borrow_mut().apply_all();

        if s.pending_undo.replace(false) {
            s.command_manager.borrow_mut().undo();
        }

        if s.pending_redo.replace(false) {
            s.command_manager.borrow_mut().redo();
        }
    });
}

/// Requests an undo for the current frame.
pub fn request_undo() {
    EDITOR_SERVICES.with(|s| s.pending_undo.set(true));
}

/// Requests an redo for the current frame.
pub fn request_redo() {
    EDITOR_SERVICES.with(|s| s.pending_redo.set(true));
}
