//! WgpuContext main struct.

use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::Window;

use super::conversions::{convert_keycode, convert_mouse_button};
use super::graphics_state::{GraphicsState, GraphicsStateError};
use super::input_state::InputState;
use super::time_state::TimeState;
use crate::types::Color;

/// Wgpu backend implementation for bishop.
pub struct WgpuContext {
    pub(crate) graphics: GraphicsState,
    pub(crate) input: InputState,
    pub(crate) time: TimeState,
    pub(crate) window: Arc<Window>,
    pub(crate) clear_color: Option<Color>,
}

impl WgpuContext {
    /// Creates a new wgpu context from a window.
    pub async fn new(window: Arc<Window>) -> Result<Self, GraphicsStateError> {
        let graphics = GraphicsState::new(window.clone()).await?;
        Ok(Self {
            graphics,
            input: InputState::new(),
            time: TimeState::new(),
            window,
            clear_color: None,
        })
    }

    /// Creates a new wgpu context synchronously using pollster.
    pub fn new_sync(window: Arc<Window>) -> Result<Self, GraphicsStateError> {
        pollster::block_on(Self::new(window))
    }

    /// Prepares for a new frame by clearing per-frame state.
    pub fn begin_frame(&mut self) {
        self.input.begin_frame();
        self.time.begin_frame();
        self.clear_color = None;
    }

    /// Processes a winit WindowEvent and updates internal state.
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.graphics.resize(size.width, size.height);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                let key = convert_keycode(*keycode);
                match state {
                    ElementState::Pressed => self.input.on_key_down(key),
                    ElementState::Released => self.input.on_key_up(key),
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if let Some(btn) = convert_mouse_button(*button) {
                    match state {
                        ElementState::Pressed => self.input.on_mouse_down(btn),
                        ElementState::Released => self.input.on_mouse_up(btn),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.on_mouse_move(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.input.on_mouse_wheel(dx, dy);
            }
            _ => {}
        }
    }

    /// Returns a reference to the underlying window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Returns the current screen width.
    pub fn screen_width(&self) -> f32 {
        self.graphics.size.0 as f32
    }

    /// Returns the current screen height.
    pub fn screen_height(&self) -> f32 {
        self.graphics.size.1 as f32
    }
}
