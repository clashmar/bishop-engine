// editor/src/editor_global.rs
use crate::commands::editor_command_manager::EditorCommandManager;
use crate::commands::editor_command_manager::EditorCommand;
use crate::gui::panels::panel_manager::PanelManager;
use std::cell::RefCell;
use std::future::Future;
use std::cell::Cell;
use std::pin::Pin;
use crate::Editor;
use std::rc::Rc;
use mlua::Lua;

/// Global services that can be reached from anywhere in the editor.
pub struct EditorServices {
    pub editor: RefCell<Option<Editor>>, // set once at startup
    pub lua: RefCell<Lua>,
    pub command_manager: RefCell<EditorCommandManager>,
    pub pending_undo: Cell<bool>,
    pub pending_redo: Cell<bool>,
    pub entity_clipboard: RefCell<Option<Vec<(String, String)>>>,
    pub panel_manager: RefCell<PanelManager>,
}

impl EditorServices {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            editor: RefCell::new(None),
            lua: RefCell::new(Lua::new()),
            command_manager: RefCell::new(EditorCommandManager::new()),
            pending_undo: Cell::new(false),
            pending_redo: Cell::new(false),
            entity_clipboard: RefCell::new(None), 
            panel_manager: RefCell::new(PanelManager::new()),
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

/// Gets immutable access to the Lua VM.
pub fn with_lua<F, R>(f: F) -> R
where
    F: FnOnce(&Lua) -> R,
{
    EDITOR_SERVICES.with(|services| {
        let lua = services.lua.borrow();
        f(&lua)
    })
}

/// Gets async immutable access to the Lua VM.
pub async fn with_lua_async<R, F>(f: F) -> R
where
    F: for<'a> FnOnce(&'a Lua) -> Pin<Box<dyn Future<Output = R> + 'a>>,
{
    let services = EDITOR_SERVICES.with(|s| s.clone());
    let lua_ref = services.lua.borrow();
    
    // Call the closure and await the future
    let future = f(&*lua_ref);
    future.await
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

/// Gets mutable access to the `PanelManager`.
pub fn with_panel_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut PanelManager) -> R,
{
    EDITOR_SERVICES.with(|services| {
        let mut pm = services.panel_manager.borrow_mut();
        f(&mut pm)
    })
}

/// Gets async mutable access to the `PanelManager`.
pub async fn with_panel_manager_async<R, F>(f: F) -> R
where
    F: for<'a> FnOnce(&'a mut PanelManager) -> Pin<Box<dyn Future<Output = R> + 'a>>,
{
    let services = EDITOR_SERVICES.with(|s| s.clone());
    let mut pm_ref = services.panel_manager.borrow_mut();
    
    // Call the closure and await the future
    let future = f(&mut*pm_ref);
    future.await
}
