//! Graphics state containing wgpu device, queue, and surface.

use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

/// Holds all wgpu graphics state.
pub struct GraphicsState {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub size: (u32, u32),
}

impl GraphicsState {
    /// Creates a new graphics state from a window.
    pub async fn new(window: Arc<Window>) -> Result<Self, GraphicsStateError> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .map_err(GraphicsStateError::SurfaceCreation)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GraphicsStateError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("bishop_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(GraphicsStateError::DeviceRequest)?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,
            size: (width, height),
        })
    }

    /// Resizes the surface to the new dimensions.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

/// Errors that can occur during graphics state initialization.
#[derive(Debug)]
pub enum GraphicsStateError {
    /// Failed to create surface from window.
    SurfaceCreation(wgpu::CreateSurfaceError),
    /// No compatible adapter found.
    NoAdapter,
    /// Failed to request device.
    DeviceRequest(wgpu::RequestDeviceError),
}

impl std::fmt::Display for GraphicsStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SurfaceCreation(e) => write!(f, "Failed to create surface: {}", e),
            Self::NoAdapter => write!(f, "No compatible graphics adapter found"),
            Self::DeviceRequest(e) => write!(f, "Failed to request device: {}", e),
        }
    }
}

impl std::error::Error for GraphicsStateError {}
