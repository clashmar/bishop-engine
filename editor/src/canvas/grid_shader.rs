// editor/src/canvas/grid_shader.rs
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
};
use once_cell::sync::OnceCell;
use bishop::prelude::*;

const GRID_VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
"#;

const GRID_FRAGMENT_SHADER: &str = include_str!("grid.frag");

struct GridResources {
    material: Material,
    texture: Texture2D,
}

static GRID_RESOURCES: OnceCell<GridResources> = OnceCell::new();

/// Initializes grid shader resources (material and texture).
fn get_grid_resources() -> Option<&'static GridResources> {
    GRID_RESOURCES.get_or_try_init(|| {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: GRID_VERTEX_SHADER,
                fragment: GRID_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("camera_pos", UniformType::Float2),
                    UniformDesc::new("camera_zoom", UniformType::Float1),
                    UniformDesc::new("viewport_size", UniformType::Float2),
                    UniformDesc::new("grid_size", UniformType::Float1),
                    UniformDesc::new("line_color", UniformType::Float4),
                    UniformDesc::new("line_thickness", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;

        // Create a 1x1 white texture for the quad
        let texture = Texture2D::from_rgba8(1, 1, &[255, 255, 255, 255]);
        texture.set_filter(FilterMode::Nearest);

        Ok::<_, macroquad::Error>(GridResources { material, texture })
    }).ok()
}

/// Parameters for drawing the shader-based grid.
pub struct GridParams {
    pub camera_pos: Vec2,
    pub camera_zoom: f32,
    pub viewport_size: Vec2,
    pub grid_size: f32,
    pub line_color: Color,
    pub line_thickness: f32,
}

/// Draws a grid using the shader-based approach.
/// Returns true if the grid was drawn, false if the resources failed to load.
pub fn draw_shader_grid(params: &GridParams) -> bool {
    let Some(resources) = get_grid_resources() else {
        return false;
    };

    let material = &resources.material;

    material.set_uniform("camera_pos", params.camera_pos);
    material.set_uniform("camera_zoom", params.camera_zoom);
    material.set_uniform("viewport_size", params.viewport_size);
    material.set_uniform("grid_size", params.grid_size);
    material.set_uniform(
        "line_color",
        vec4(
            params.line_color.r,
            params.line_color.g,
            params.line_color.b,
            params.line_color.a,
        ),
    );
    material.set_uniform("line_thickness", params.line_thickness);

    gl_use_material(material);

    // Draw a full-screen quad in world space that covers the viewport
    let half_w = params.viewport_size.x / params.camera_zoom * 0.5;
    let half_h = params.viewport_size.y / params.camera_zoom * 0.5;

    draw_texture_ex(
        &resources.texture,
        params.camera_pos.x - half_w,
        params.camera_pos.y - half_h,
        Color::WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(half_w * 2.0, half_h * 2.0)),
            ..Default::default()
        },
    );

    gl_use_default_material();
    true
}
