//! Fullscreen quad renderer for post-processing passes.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::uniforms::{CameraUniforms, ModelUniforms};

/// Vertex for fullscreen quad rendering with position and UV.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FullscreenVertex {
    /// Position in 3D space (z typically 0).
    pub position: [f32; 3],
    /// Texture coordinates.
    pub tex_coord: [f32; 2],
}

impl FullscreenVertex {
    /// Returns the vertex buffer layout for this vertex type.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Renderer for fullscreen quads used in post-processing passes.
/// Provides a reusable vertex buffer and camera/model uniforms for material passes.
pub struct FullscreenQuadRenderer {
    vertex_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    model_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl FullscreenQuadRenderer {
    /// Creates a new fullscreen quad renderer.
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = Self::create_fullscreen_vertices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen_quad_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen_quad_camera_buffer"),
            contents: bytemuck::cast_slice(&[CameraUniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen_quad_model_buffer"),
            contents: bytemuck::cast_slice(&[ModelUniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fullscreen_quad_camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fullscreen_quad_camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            vertex_buffer,
            camera_buffer,
            model_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        }
    }

    /// Updates the camera uniforms for render target rendering.
    pub fn update_camera(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        let camera = CameraUniforms::screen_space(width, height);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera]));
    }

    /// Updates the model uniforms for vertex transformation.
    pub fn update_model(&self, queue: &wgpu::Queue, model: &ModelUniforms) {
        queue.write_buffer(&self.model_buffer, 0, bytemuck::cast_slice(&[*model]));
    }

    /// Sets up the render pass for drawing a fullscreen quad.
    /// Binds the vertex buffer and camera bind group.
    pub fn prepare_pass<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
    }

    /// Draws the fullscreen quad.
    /// Assumes prepare_pass has been called and pipeline + other bind groups are set.
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.draw(0..6, 0..1);
    }

    /// Returns the camera bind group layout for pipeline creation.
    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    /// Returns the camera bind group for rendering.
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    /// Returns the vertex buffer layout for pipeline creation.
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        FullscreenVertex::layout()
    }

    fn create_fullscreen_vertices() -> [FullscreenVertex; 6] {
        [
            FullscreenVertex {
                position: [0.0, 0.0, 0.0],
                tex_coord: [0.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, 0.0, 0.0],
                tex_coord: [1.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, 1.0, 0.0],
                tex_coord: [1.0, 1.0],
            },
            FullscreenVertex {
                position: [0.0, 0.0, 0.0],
                tex_coord: [0.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, 1.0, 0.0],
                tex_coord: [1.0, 1.0],
            },
            FullscreenVertex {
                position: [0.0, 1.0, 0.0],
                tex_coord: [0.0, 1.0],
            },
        ]
    }
}
