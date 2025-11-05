// editor/src/global.rs
use std::{
    cell::{
        Cell, 
        RefCell
    }, 
    future::Future, 
    pin::Pin, 
    rc::Rc
};
use crate::{
    commands::command_manager::{
        Command, 
        CommandManager
    }, 
    editor::Editor
};

/// Global services that can be reached from anywhere in the editor.
pub struct Services {
    pub editor: RefCell<Option<Editor>>, // set once at startup
    pub command_manager: RefCell<CommandManager>,
    pub pending_undo: Cell<bool>,
    pub pending_redo: Cell<bool>,
    pub entity_clipboard: RefCell<Option<Vec<(String, String)>>>,
}

impl Services {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            editor: RefCell::new(None),
            command_manager: RefCell::new(CommandManager::new()),
            pending_undo: Cell::new(false),
            pending_redo: Cell::new(false),
            entity_clipboard: RefCell::new(None), 
        })
    }
}

/// Store the `Editor` in global services.
pub fn set_editor(editor: Editor) {
    SERVICES.with(|services| {
        *services.editor.borrow_mut() = Some(editor);
    });
}

thread_local! {
    /// Single instance of services used by the whole program.
    pub static SERVICES: Rc<Services> = Services::new();
}

pub fn with_editor<F, R>(f: F) -> R
where
    F: FnOnce(&mut Editor) -> R,
{
    SERVICES.with(|services| {
        let mut opt = services.editor.borrow_mut();
        let editor = opt
            .as_mut()
            .expect("Editor not initialised");
        f(editor)
    })
}

pub async fn with_editor_async<R, F>(f: F) -> R
where
    for<'a> F: FnOnce(&'a mut Editor) -> Pin<Box<dyn Future<Output = R> + 'a>>,
{
    let services = SERVICES.with(|s| s.clone());

    let mut opt = services.editor.borrow_mut();
    let editor = opt
        .as_mut()
        .expect("Editor not initialised");

    let fut = f(editor);
    fut.await
}

pub fn push_command(command: Box<dyn Command>) {
    SERVICES.with(|services| {
        services.command_manager.borrow_mut().push(command);
    });
}

pub fn apply_pending_commands() {
    SERVICES.with(|s| {
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

pub fn request_undo() {
    SERVICES.with(|s| s.pending_undo.set(true));
}
pub fn request_redo() {
    SERVICES.with(|s| s.pending_redo.set(true));
}
