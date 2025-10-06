// editor/src/controls/command_manager.rs

use std::fmt::Debug;

/// Trait for every undoable command.
pub trait Command: Debug {
    fn execute(&mut self);
    fn undo(&mut self);
}
/// Stores and manages undo/redo stacks.
pub struct CommandManager {
    pending: Vec<Box<dyn Command>>,
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandManager {
    /// Returns a new undo and redo stack.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Push the command to the pending stack to be executed safely at the end of the frame.
    pub fn push(&mut self, command: Box<dyn Command>) {
        self.redo_stack.clear();
        self.pending.push(command);
    }

    /// Undo a command on the undo stack and push it onto the redo stack.
    pub fn undo(&mut self) {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo();
            self.redo_stack.push(command);
        }
    }

    /// Redo a command on the redo stack and push it onto the undo stack.
    pub fn redo(&mut self) {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute();
            self.undo_stack.push(command);
        }
    }

    /// Execute all commands that have been queued this frame.
    pub fn apply_all(&mut self) {
        while let Some(mut command) = self.pending.pop() {
            command.execute();
            self.undo_stack.push(command);
           
        }
    }
}

