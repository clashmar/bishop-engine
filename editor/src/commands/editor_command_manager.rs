// editor/src/controls/editor_command_manager.rs
use std::fmt::Debug;
use crate::{editor::EditorMode, editor_global::with_editor};

/// Trait for every undoable command.
pub trait EditorCommand: Debug {
    fn execute(&mut self);
    fn undo(&mut self);
    fn mode(&self) -> EditorMode;
}

/// Stores and manages undo/redo stacks.
pub struct EditorCommandManager {
    pending: Vec<Box<dyn EditorCommand>>,
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
}

impl EditorCommandManager {
    /// Returns a new undo and redo stack.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Push the command to the pending stack to be executed safely at the end of the frame.
    pub fn push(&mut self, command: Box<dyn EditorCommand>) {
        self.redo_stack.clear();
        self.pending.push(command);
    }

    /// Undo a command on the undo stack and push it onto the redo stack.
    pub fn undo(&mut self) {
        // Get the current editor mode
        let current_mode = with_editor(|editor| editor.mode);

        // Temp buffer
        let mut buffer: Vec<Box<dyn EditorCommand>> = Vec::new();

        // Find the first command that matches the current mode
        while let Some(mut command) = self.undo_stack.pop() {
            if command.mode() == current_mode {
                command.undo();
                self.redo_stack.push(command);
                break;
            } else {
                // Push non matching commands to the temp
                buffer.push(command);
            }
        }

        // Push the temp back to the undo stack
        while let Some(command) = buffer.pop() {
            self.undo_stack.push(command);
        }
    }

    /// Redo a command on the redo stack and push it onto the undo stack.
    pub fn redo(&mut self) {
        // Get the current editor mode
        let current_mode = with_editor(|editor| editor.mode);

        // Temp buffer
        let mut buffer: Vec<Box<dyn EditorCommand>> = Vec::new();

        // Find the first command that matches the current mode
        while let Some(mut cmd) = self.redo_stack.pop() {
            if cmd.mode() == current_mode {
                cmd.execute();
                self.undo_stack.push(cmd);
                break;
            } else {
                buffer.push(cmd);
            }
        }

        // Push the temp back to the redo stack
        while let Some(cmd) = buffer.pop() {
            self.redo_stack.push(cmd);
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

