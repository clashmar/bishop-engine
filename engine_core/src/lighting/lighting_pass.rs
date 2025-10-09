use macroquad::{miniquad::PassAction, prelude::*};
use crate::lighting::light::LightUniform;

const MAX_LIGHTS: usize = 64;

pub struct LightingPass {
    material: Material,
    target: RenderTarget,
    screen_w: u32,
    screen_h: u32,
}

impl LightingPass {
    pub async fn new(screen_w: u32, screen_h: u32) -> Self {
        let target = render_target(screen_w, screen_h);
        target.texture.set_filter(FilterMode::Linear);

        let uniform_descs = vec![
            UniformDesc::new("screen_size", UniformType::Float2),
            UniformDesc::new("light_count", UniformType::Int1),
            UniformDesc::new("light_pos_radius", UniformType::Float3).array(MAX_LIGHTS as usize),
            UniformDesc::new("light_colour", UniformType::Float3).array(MAX_LIGHTS as usize),
            UniformDesc::new("cam_target", UniformType::Float2),
            UniformDesc::new("cam_zoom", UniformType::Float2),
        ];

        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                textures: vec![("scene".to_string())],
                uniforms: uniform_descs,
                ..Default::default()
            },
        )
        .expect("Failed to compile lighting shader");

        let scene_tex = target.texture.clone();
        material.set_texture("scene", scene_tex);

        Self { 
            material, 
            target,
            screen_w,
            screen_h, 
        }
    }

    /// Call before you start drawing the world.
    pub fn begin_scene(&self, camera: &Camera2D) {
        set_camera(camera);
        
        let ctx = unsafe { get_internal_gl().quad_context };

        ctx.begin_pass(
            Some(self.target.render_pass.raw_miniquad_id()), 
            PassAction::clear_color(0.0, 0.0, 0.0, 1.0)
        );
    }

    pub fn end_scene(&mut self, lights: &[LightUniform], camera: &Camera2D) {
        let ctx = unsafe { get_internal_gl().quad_context };
        ctx.end_render_pass();               

        ctx.begin_default_pass(PassAction::Nothing);

        self.material.set_uniform("screen_size", vec2(self.screen_w as f32, self.screen_h as f32));
        self.material.set_uniform("light_count", lights.len() as i32);

        let mut flat_pos_radius = vec![0.0_f32; MAX_LIGHTS * 3];
        let mut flat_colour = vec![0.0_f32; MAX_LIGHTS * 3];

        for (i, l) in lights.iter().enumerate() {
            flat_pos_radius[i * 3..i * 3 + 3].copy_from_slice(&l.pos_radius);
            flat_colour[i * 3..i * 3 + 3].copy_from_slice(&l.colour);
        }

        self.material.set_uniform_array("light_pos_radius", &flat_pos_radius);
        self.material.set_uniform_array("light_colour", &flat_colour);

        self.material.set_uniform("cam_target", camera.target);
        self.material.set_uniform("cam_zoom", camera.zoom);

        gl_use_material(&self.material);

        self.material.set_texture("scene", self.target.texture.clone());

        draw_texture_ex(
            &self.target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        gl_use_default_material();
        ctx.end_render_pass();

        
    }
}


pub const VERTEX: &str = r#"
#version 100
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
precision lowp float;

varying vec2 v_uv;
uniform sampler2D scene;
uniform vec2 screen_size;
uniform vec2 cam_target;
uniform vec2 cam_zoom;  
uniform int light_count;
uniform vec3 light_pos_radius[64]; // xyz = (world_x, world_y, radius)
uniform vec3 light_colour[64]; // rgb * intensity

vec2 uv_to_world(vec2 uv) {
    // 0‑1 → pixel coordinates
    vec2 pixel = uv * screen_size;

    // pixel → world (undo the camera transform)
    //   world = (pixel - screen_center) / cam_zoom + cam_target
    return (pixel - screen_size * 0.5) / cam_zoom + cam_target;
}

void main() {
    // Base colour is the dark background
    vec3 result = vec3(0.0); // black

    vec2 world = uv_to_world(v_uv);

    // Accumulate every light
    for (int i = 0; i < light_count; ++i) {
        vec3 pr = light_pos_radius[i];
        vec2  light_pos = pr.xy;
        float radius    = pr.z;

        // distance in world units
        float dist = length(world - light_pos);
        // Simple linear fall‑off (clamp to [0,1])
        float att = clamp(1.0 - dist / radius, 0.0, 1.0);

        result += light_colour[i] * att;
    }

    // Multiply the lit colour with the scene texture (optional)
    vec3 scene_col = texture2D(scene, v_uv).rgb;
    result = mix(scene_col * 0.2, scene_col, result); // 0.2 = ambient darkness

    gl_FragColor = vec4(result, 1.0);
}
"#;