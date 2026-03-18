// Ambient shader for darkness application.
// Darkens the scene based on the darkness uniform value.

struct AmbientUniforms {
    darkness: f32,
    _pad: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> params: AmbientUniforms;

@group(2) @binding(0)
var t_scene: texture_2d<f32>;

@group(2) @binding(1)
var s_scene: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(t_scene, s_scene, in.uv);
    let scene = base.rgb;

    let darkened = mix(scene, vec3<f32>(0.0), params.darkness);

    return vec4<f32>(clamp(darkened, vec3<f32>(0.0), vec3<f32>(1.0)), base.a);
}
