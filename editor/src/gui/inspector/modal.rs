// editor/src/gui/inspector/modal.rs
use engine_core::assets::asset_manager::AssetManager;
use macroquad::prelude::*;

pub struct Modal {
    /// Position & size of the modal window.
    pub rect: Rect,
    pub open: bool,
    draw_callback: DrawCallback,
    just_opened: bool,
}

type BoxedDraw = Box<dyn FnMut(&mut AssetManager) + 'static>;
type DrawCallback = Option<BoxedDraw>;

impl Modal {
    /// Creates a new modal of the given size. It is automatically centered.
    pub fn new(width: f32, height: f32) -> Self {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let rect = Rect::new(
            (screen_w - width) / 2.0,
            (screen_h - height) / 2.0,
            width,
            height,
        );
        Self {
            rect,
            open: false,
            draw_callback: None,
            just_opened: false,
        }
    }

    /// Open the modal and set draw callbacks.
    pub fn open<F>(&mut self, draw_content: F)
    where
        F: FnMut(&mut AssetManager) + 'static,
    {
        self.open = true;
        self.draw_callback = Some(Box::new(draw_content));
        self.just_opened = true; 
    }

    /// Close the modal.
    pub fn close(&mut self) {
        self.open = false;
        self.draw_callback = None;
    }

    /// Returns `true` if the modal is currently open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Render the modal. Returns `true`` when the user clicked outside the window.
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

        // Optional title?

        // Run the callbacks
        if let Some(callback) = self.draw_callback.as_mut() {
            callback.as_mut()(asset_manager);
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
}