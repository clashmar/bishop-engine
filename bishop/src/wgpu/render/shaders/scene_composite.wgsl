// Scene composite shader for ambient + glow blend.
// Combines the ambient-lit scene with glow effects using additive RGB blending.

@group(2) @binding(0)
var t_ambient: texture_2d<f32>;

@group(2) @binding(1)
var s_ambient: sampler;

@group(2) @binding(2)
var t_glow: texture_2d<f32>;

@group(2) @binding(3)
var s_glow: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let amb = textureSample(t_ambient, s_ambient, in.uv);
    let glow = textureSample(t_glow, s_glow, in.uv);

    let src_a = amb.a;
    let src_rgb = amb.rgb + glow.rgb;

    return vec4<f32>(src_rgb, src_a);
}
