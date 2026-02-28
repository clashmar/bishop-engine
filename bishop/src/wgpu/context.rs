//! WgpuContext main struct.

use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::{Fullscreen, Window};

use super::conversions::{convert_keycode, convert_mouse_button, keycode_to_char};
use super::conversions_window::convert_cursor_icon;
use super::exec::FrameFuture;
use super::graphics_state::{GraphicsState, GraphicsStateError};
use super::input_state::InputState;
use super::render::{
    create_texture_bind_group_layout, BishopRenderTarget, CameraUniforms, FullscreenQuadRenderer,
    PrimitiveRenderer, TextRenderer, TextureRenderer,
};
use super::texture_loader::init_texture_loader;
use super::time_state::TimeState;
use crate::camera::Camera2D;
use crate::types::Color;
use crate::window::CursorIcon;

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
    render_target_bind_group_layout: std::sync::Arc<wgpu::BindGroupLayout>,
    fullscreen_quad_renderer: FullscreenQuadRenderer,
    fullscreen: bool,
    scale_factor: f32,
    current_surface_texture: Option<wgpu::SurfaceTexture>,
    current_surface_view: Option<wgpu::TextureView>,
    has_cleared_this_frame: bool,
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

        // Initialize the global texture loader for free function texture loading
        init_texture_loader(
            graphics.device.clone(),
            graphics.queue.clone(),
            texture_renderer.texture_bind_group_layout_arc(),
        );

        let render_target_bind_group_layout = std::sync::Arc::new(create_texture_bind_group_layout(
            &graphics.device,
        ));
        let fullscreen_quad_renderer = FullscreenQuadRenderer::new(&graphics.device);
        let scale_factor = window.scale_factor() as f32;
        let fullscreen = window.fullscreen().is_some();

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
            render_target_bind_group_layout,
            fullscreen_quad_renderer,
            fullscreen,
            scale_factor,
            current_surface_texture: None,
            current_surface_view: None,
            has_cleared_this_frame: false,
        })
    }

    /// Creates a new wgpu context synchronously using pollster.
    pub fn new_sync(window: Arc<Window>) -> Result<Self, GraphicsStateError> {
        pollster::block_on(Self::new(window))
    }

    /// Prepares for a new frame by clearing per-frame state.
    pub fn begin_frame(&mut self) {
        match self.graphics.surface.get_current_texture() {
            Ok(texture) => {
                let view = texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.current_surface_texture = Some(texture);
                self.current_surface_view = Some(view);
            }
            Err(e) => {
                eprintln!("Failed to acquire surface texture: {e}");
                self.current_surface_texture = None;
                self.current_surface_view = None;
            }
        }

        // Now measure timing
        self.input.begin_frame();
        self.time.begin_frame();

        // Clear state
        self.clear_color = None;
        self.primitive_renderer.clear();
        self.texture_renderer.clear();
        self.text_renderer.clear();
        self.current_camera = None;
        self.has_cleared_this_frame = false;
    }

    /// Processes a winit WindowEvent and updates internal state.
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.graphics.resize(size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = *scale_factor as f32;
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
                let x = position.x as f32;
                let y = position.y as f32;
                self.input.on_mouse_move(x, y);
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

    /// Returns the current screen width in physical pixels.
    pub fn screen_width(&self) -> f32 {
        self.graphics.size.0 as f32
    }

    /// Returns the current screen height in physical pixels.
    pub fn screen_height(&self) -> f32 {
        self.graphics.size.1 as f32
    }

    /// Sets the mouse cursor icon.
    pub fn set_cursor_icon(&mut self, icon: CursorIcon) {
        self.window.set_cursor(convert_cursor_icon(icon));
    }

    /// Toggles fullscreen mode and returns the new state.
    pub fn toggle_fullscreen(&mut self) -> bool {
        self.fullscreen = !self.fullscreen;
        let fullscreen_mode = if self.fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        };
        self.window.set_fullscreen(fullscreen_mode);
        self.fullscreen
    }

    /// Returns whether the window is currently in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Returns the display scale factor (DPI scaling).
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Returns the texture bind group layout for creating textures.
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.texture_renderer.texture_bind_group_layout()
    }

    /// Returns a reference to the wgpu device.
    pub fn device(&self) -> &wgpu::Device {
        &self.graphics.device
    }

    /// Returns the Arc-wrapped wgpu device for shared ownership.
    pub fn device_arc(&self) -> Arc<wgpu::Device> {
        self.graphics.device.clone()
    }

    /// Returns a reference to the wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.graphics.queue
    }

    /// Returns the Arc-wrapped wgpu queue for shared ownership.
    pub fn queue_arc(&self) -> Arc<wgpu::Queue> {
        self.graphics.queue.clone()
    }

    /// Returns the Arc-wrapped texture bind group layout for shared ownership.
    pub fn texture_bind_group_layout_arc(&self) -> Arc<wgpu::BindGroupLayout> {
        self.texture_renderer.texture_bind_group_layout_arc()
    }

    /// Returns the surface format for pipeline creation.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.graphics.config.format
    }

    /// Creates a render target with the specified dimensions.
    pub fn create_render_target(&self, width: u32, height: u32) -> BishopRenderTarget {
        BishopRenderTarget::new(
            &self.graphics.device,
            self.render_target_bind_group_layout.clone(),
            width,
            height,
            self.graphics.config.format
        )
    }

    /// Returns the render target bind group layout for creating render targets externally.
    pub fn render_target_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.render_target_bind_group_layout
    }

    /// Returns the Arc-wrapped render target bind group layout for shared ownership.
    pub fn render_target_bind_group_layout_arc(&self) -> Arc<wgpu::BindGroupLayout> {
        self.render_target_bind_group_layout.clone()
    }

    /// Returns a reference to the fullscreen quad renderer.
    pub fn fullscreen_quad_renderer(&self) -> &FullscreenQuadRenderer {
        &self.fullscreen_quad_renderer
    }

    /// Returns a future that completes on the next frame.
    /// Use this to yield from async code and allow the frame to render.
    pub fn next_frame(&self) -> FrameFuture {
        FrameFuture::new()
    }

    /// Clears per-frame input state (pressed/released events).
    /// Called by app_runner when a frame completes.
    pub fn end_frame_input(&mut self) {
        self.input.end_frame();
    }

    /// Returns true if any renderer has pending draw calls.
    fn has_pending_draws(&self) -> bool {
        !self.primitive_renderer.is_empty()
            || !self.texture_renderer.is_empty()
            || !self.text_renderer.is_empty()
    }

    /// Flushes batched draw calls with the current camera state.
    fn flush_batched(&mut self, view: &wgpu::TextureView, load_op: wgpu::LoadOp<wgpu::Color>) {
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
                    label: Some("flush_encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("flush_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
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

        self.primitive_renderer.clear();
        self.texture_renderer.clear();
        self.text_renderer.clear();
    }

    /// Flushes pending draws if there are any batched vertices.
    /// Called before camera changes to ensure draws use the correct transform.
    pub(crate) fn flush_if_needed(&mut self) {
        if !self.has_pending_draws() {
            return;
        }

        let Some(view) = self.current_surface_view.take() else {
            return;
        };

        let load_op = if self.has_cleared_this_frame {
            wgpu::LoadOp::Load
        } else {
            let clear_color = self.clear_color.unwrap_or(Color::BLACK);
            self.has_cleared_this_frame = true;
            wgpu::LoadOp::Clear(wgpu::Color {
                r: clear_color.r as f64,
                g: clear_color.g as f64,
                b: clear_color.b as f64,
                a: clear_color.a as f64,
            })
        };

        self.flush_batched(&view, load_op);

        self.current_surface_view = Some(view);
    }

    /// Renders the current frame and presents it.
    pub fn render_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.flush_if_needed();

        if let Some(texture) = self.current_surface_texture.take() {
            texture.present();
        }
        self.current_surface_view = None;

        Ok(())
    }
}
