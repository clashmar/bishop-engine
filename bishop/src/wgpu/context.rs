//! WgpuContext main struct.

use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::Window;

use super::conversions::{convert_keycode, convert_mouse_button, keycode_to_char};
use super::graphics_state::{GraphicsState, GraphicsStateError};
use super::input_state::InputState;
use super::render::{CameraUniforms, PrimitiveRenderer, TextRenderer, TextureRenderer};
use super::time_state::TimeState;
use crate::camera::Camera2D;
use crate::types::Color;

/// Wgpu backend implementation for bishop.
pub struct WgpuContext {
    pub(crate) graphics: GraphicsState,
    pub(crate) input: InputState,
    pub(crate) time: TimeState,
    pub(crate) window: Arc<Window>,
    pub(crate) clear_color: Option<Color>,
    pub(crate) primitive_renderer: PrimitiveRenderer,
    pub(crate) texture_renderer: TextureRenderer,
    pub(crate) text_renderer: TextRenderer,
    pub(crate) current_camera: Option<Camera2D>,
}

impl WgpuContext {
    /// Creates a new wgpu context from a window.
    pub async fn new(window: Arc<Window>) -> Result<Self, GraphicsStateError> {
        let graphics = GraphicsState::new(window.clone()).await?;

        let primitive_renderer =
            PrimitiveRenderer::new(&graphics.device, graphics.config.format);
        let texture_renderer = TextureRenderer::new(&graphics.device, graphics.config.format);
        let text_renderer =
            TextRenderer::new(&graphics.device, &graphics.queue, graphics.config.format);

        Ok(Self {
            graphics,
            input: InputState::new(),
            time: TimeState::new(),
            window,
            clear_color: None,
            primitive_renderer,
            texture_renderer,
            text_renderer,
            current_camera: None,
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
        self.primitive_renderer.clear();
        self.texture_renderer.clear();
        self.text_renderer.clear();
        self.current_camera = None;
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
                        text,
                        ..
                    },
                ..
            } => {
                let key = convert_keycode(*keycode);
                match state {
                    ElementState::Pressed => {
                        self.input.on_key_down(key);
                        let mut got_text = false;
                        if let Some(txt) = text {
                            for c in txt.chars() {
                                if !c.is_control() {
                                    self.input.on_char(c);
                                    got_text = true;
                                }
                            }
                        }
                        if !got_text {
                            let shift = self.input.is_key_down(crate::input::KeyCode::LeftShift)
                                || self.input.is_key_down(crate::input::KeyCode::RightShift);
                            if let Some(c) = keycode_to_char(*keycode, shift) {
                                self.input.on_char(c);
                            }
                        }
                    }
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

    /// Returns the texture bind group layout for creating textures.
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.texture_renderer.texture_bind_group_layout()
    }

    /// Returns a reference to the wgpu device.
    pub fn device(&self) -> &wgpu::Device {
        &self.graphics.device
    }

    /// Returns a reference to the wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.graphics.queue
    }

    /// Renders the current frame and presents it.
    pub fn render_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.graphics.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let width = self.graphics.size.0 as f32;
        let height = self.graphics.size.1 as f32;

        let uniforms = if let Some(camera) = &self.current_camera {
            CameraUniforms::from_camera2d(camera, width, height)
        } else {
            CameraUniforms::screen_space(width, height)
        };

        self.primitive_renderer
            .update_uniforms(&self.graphics.queue, &uniforms);
        self.texture_renderer
            .update_uniforms(&self.graphics.queue, &uniforms);
        self.text_renderer
            .update_uniforms(&self.graphics.queue, &uniforms);

        let mut encoder =
            self.graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

        {
            let clear_color = self.clear_color.unwrap_or(Color::BLACK);
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.primitive_renderer
                .flush(&self.graphics.queue, &mut render_pass);
            self.texture_renderer
                .flush(&self.graphics.queue, &mut render_pass);
            self.text_renderer
                .flush(&self.graphics.queue, &mut render_pass);
        }

        self.graphics.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
