//! Internal application runner for wgpu backend.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window, WindowId};

use super::context::WgpuContext;
use crate::time::Time;
use crate::window::{IconData, WindowConfig, WindowIcon};
use crate::BishopApp;

/// Internal application handler for running BishopApp with wgpu/winit.
pub(crate) struct WgpuAppRunner<A: BishopApp> {
    config: WindowConfig,
    app: A,
    ctx: Option<WgpuContext>,
    window: Option<Arc<Window>>,
}

impl<A: BishopApp> WgpuAppRunner<A> {
    /// Creates a new app runner with the given config and app.
    pub fn new(config: WindowConfig, app: A) -> Self {
        Self {
            config,
            app,
            ctx: None,
            window: None,
        }
    }
}

impl<A: BishopApp> ApplicationHandler for WgpuAppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let fullscreen = if self.config.fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        };

        let mut attrs = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable)
            .with_fullscreen(fullscreen);

        if let Some(icon) = &self.config.icon {
            if let Some(winit_icon) = convert_window_icon(icon) {
                attrs = attrs.with_window_icon(Some(winit_icon));
            }
        }

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        match WgpuContext::new_sync(window.clone()) {
            Ok(ctx) => self.ctx = Some(ctx),
            Err(e) => {
                eprintln!("Failed to create WgpuContext: {e}");
                event_loop.exit();
                return;
            }
        }

        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(ctx) = &mut self.ctx {
            ctx.handle_window_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(ctx) = &mut self.ctx {
                    ctx.begin_frame();
                    pollster::block_on(self.app.frame(ctx));
                    if let Err(e) = ctx.render_frame() {
                        eprintln!("Render error: {e}");
                    }
                    ctx.update();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Converts a WindowIcon to a winit Icon.
fn convert_window_icon(icon: &WindowIcon) -> Option<winit::window::Icon> {
    match icon {
        WindowIcon::Png(data) => {
            let img = image::load_from_memory(data).ok()?.to_rgba8();
            let (width, height) = img.dimensions();
            winit::window::Icon::from_rgba(img.into_raw(), width, height).ok()
        }
        WindowIcon::Rgba { small, medium, large } => {
            let icon_data = large.as_ref().or(medium.as_ref()).or(small.as_ref())?;
            create_icon_from_data(icon_data)
        }
    }
}

/// Creates a winit Icon from IconData.
fn create_icon_from_data(data: &IconData) -> Option<winit::window::Icon> {
    winit::window::Icon::from_rgba(data.rgba.clone(), data.width, data.height).ok()
}
