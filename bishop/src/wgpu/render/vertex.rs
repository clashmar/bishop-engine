//! Vertex types for 2D rendering.

use bytemuck::{Pod, Zeroable};

/// Vertex for colored primitives (rectangles, circles, lines, triangles).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PrimitiveVertex {
    /// Position in screen space.
    pub position: [f32; 2],
    /// RGBA color.
    pub color: [f32; 4],
}

impl PrimitiveVertex {
    /// Creates a new primitive vertex.
    pub const fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, color }
    }

    /// Returns the vertex buffer layout for this vertex type.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Vertex for textured rendering (sprites, text).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TexturedVertex {
    /// Position in screen space.
    pub position: [f32; 2],
    /// Texture coordinates.
    pub tex_coord: [f32; 2],
    /// RGBA color tint.
    pub color: [f32; 4],
}

impl TexturedVertex {
    /// Creates a new textured vertex.
    pub const fn new(position: [f32; 2], tex_coord: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            tex_coord,
            color,
        }
    }

    /// Returns the vertex buffer layout for this vertex type.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
