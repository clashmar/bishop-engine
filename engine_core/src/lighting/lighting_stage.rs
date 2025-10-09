// engine_core/src/lighting/lighting_stage.rs
use macroquad::miniquad::*;

pub struct Stage {
    pub pipeline: Pipeline,
    pub bindings: Bindings,
 }

impl Stage {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        #[rustfmt::skip]
        let vertices = [
            // pos        uv
            (-0.5, -0.5, 0.0, 0.0),
            ( 0.5, -0.5, 1.0, 0.0),
            ( 0.5,  0.5, 1.0, 1.0),
            (-0.5,  0.5, 0.0, 1.0),
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&[0u16, 1, 2, 0, 2, 3][..]),
        );

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                meta(),
            )
            .expect("Failed to compile lighting shader");

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv",  VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                // we want additive blending for the lights
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::One,
                )),
                ..Default::default()
            },
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        Self { pipeline, bindings }
    }
}

fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["scene".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("light_count", UniformType::Int1),
                UniformDesc::new("light_position", UniformType::Float2).array(8),
                UniformDesc::new("light_radius", UniformType::Float1).array(8),
                UniformDesc::new("light_intensity", UniformType::Float1).array(8),
                UniformDesc::new("light_colour", UniformType::Float4).array(8),
                UniformDesc::new("scene_size", UniformType::Float2), 
            ],
        },
    }
}

/// Rust side representation. Must match the layout in the shader.
#[repr(C)]
pub struct Uniforms {
    pub light_count: i32,
    pub light_position: [(f32, f32); 8],
    pub light_radius: [f32; 8],
    pub light_intensity: [f32; 8],
    pub light_colour: [(f32, f32, f32, f32); 8],
    pub scene_size: (f32, f32),  
}

pub const VERTEX: &str = r#"
#version 100
precision mediump float;

attribute vec2 pos;
attribute vec2 uv;
varying vec2 v_uv;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    v_uv = uv;
}
"#;

pub const FRAGMENT: &str = r#"
#version 100
precision mediump float;
varying vec2 v_uv;
uniform sampler2D scene;
uniform vec2 scene_size;          // not used for lighting any more, but kept for possible future use

const int MAX_LIGHTS = 8;
uniform int  light_count;
uniform vec2  light_position[MAX_LIGHTS];   // now **pixel** coordinates
uniform float light_radius[MAX_LIGHTS];     // now **pixel** radius
uniform float light_intensity[MAX_LIGHTS];
uniform vec4  light_colour[MAX_LIGHTS];

float attenuation(vec2 p, vec2 lpos, float rad) {
    float d = distance(p, lpos);
    return max(0.0, 1.0 - d / rad);
}

void main() {
    vec4 base = texture2D(scene, v_uv);
    // Convert UV to pixel coordinates (same as before)
    vec2 screen_pos = v_uv * scene_size;

    vec3 col = base.rgb;
    for (int i = 0; i < light_count; ++i) {
        float att = attenuation(screen_pos,
                                light_position[i],
                                light_radius[i]);
        col += light_colour[i].rgb * att * light_intensity[i];
    }
    gl_FragColor = vec4(col, base.a);
}
"#;