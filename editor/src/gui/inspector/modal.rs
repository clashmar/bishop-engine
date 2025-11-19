// editor/src/gui/inspector/modal.rs
use std::{cell::RefCell, thread::LocalKey};
use engine_core::{assets::asset_manager::AssetManager, ui::prompt::*};
use macroquad::prelude::*;

#[derive(Default)]
pub struct Modal {
    /// Position & size of the modal window.
    pub rect: Rect,
    pub open: bool,
    widgets: BoxedWidgets,
    just_opened: bool,
}

thread_local! {
    pub static MODAL_OPEN: RefCell<bool> = RefCell::new(false);
}

/// Global flag that tells the rest of the editor whether a modal
/// is currently open.
pub fn is_modal_open() -> bool {
    MODAL_OPEN.with(|f| *f.borrow())
}

pub type BoxedWidget = Box<dyn FnMut(&mut AssetManager) + 'static>;
type BoxedWidgets = Vec<BoxedWidget>;

/// Used by callers of a a modal to decide what should happen if 
/// the user clicks outside the modal. 
#[derive(Clone, PartialEq)]
pub enum ModalResult {
    String(String),
    ClickedOutside,
}

impl Modal {
    /// Creates a new modal of the given size. It is automatically centered.
    pub fn new(width: f32, height: f32) -> Self {
        let rect = Rect::new(
            (screen_width() - width) / 2.0,
            (screen_height() - height) / 2.0,
            width,
            height,
        );

        Self {
            rect,
            open: false,
            widgets: Vec::new(),
            just_opened: false,
        }
    }

    /// Open the modal and set draw callbacks.
    pub fn open(&mut self, callbacks: Vec<BoxedWidget>) {
        self.open = true;
        self.widgets = callbacks;
        self.just_opened = true; 

        // Let the editor know a modal is open
        MODAL_OPEN.with(|r| {
            *r.borrow_mut() = true;
        });    
    }

    /// Close the modal.
    pub fn close(&mut self) {
        self.open = false;
        self.widgets = Vec::new();

        // Let the editor know the modal is close
        MODAL_OPEN.with(|r| {
            *r.borrow_mut() = false;
        });  
    }

    /// Returns `true` if the modal is currently open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Render the modal. Returns `true`` when the user clicked outside the window.
    /// Needs asset manager for widgets that need to access assets.
    pub fn draw(&mut self, asset_manager: &mut AssetManager) -> bool {
        if !self.open {
            return false;
        }

        // Dim the whole screen
        draw_rectangle(
            0.0, 
            0.0, 
            screen_width(), 
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.6)
        );

        // Window background & outline
        draw_rectangle(
            self.rect.x, 
            self.rect.y, 
            self.rect.w, 
            self.rect.h,
            Color::new(0.08, 0.08, 0.10, 0.95)
        );

        draw_rectangle_lines(
            self.rect.x, 
            self.rect.y, 
            self.rect.w, 
            self.rect.h,
            2.0, 
            WHITE
        );

        // Run all widgets
        for widget in self.widgets.iter_mut() {
            widget.as_mut()(asset_manager);
        }

        // Skip the outside click check if just opened
        if self.just_opened {
            self.just_opened = false;
            return false;
        }

        // Detect a click outside the window
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = mouse_position().into();
            if !self.rect.contains(mouse) {
                return true;
            }
        }

        false
    }

    /// Opens a model with a confirm prompt widget.
    /// The caller must pass in a static reference to the result store.
    pub fn open_confirm_modal(result_store: &'static LocalKey<RefCell<Option<ConfirmPromptResult>>>) -> Modal {
        let prompt_message = "Are You Sure?";
        let mut modal = Modal::new(300.0, 120.0);

        let mut prompt = ConfirmPromptWidget::new(modal.rect, prompt_message);

        let widgets: Vec<BoxedWidget> = vec![ 
            Box::new(move |_| {
                if let Some(result) = prompt.draw() {
                    // Write the result to the static thread local
                    result_store.with(|c| *c.borrow_mut() = Some(result));
                }
            })
        ];

        modal.open(widgets);
        modal
    }
}



