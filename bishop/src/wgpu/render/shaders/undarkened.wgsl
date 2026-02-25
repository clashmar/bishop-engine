// Undarkened shader for glow blend on lighting input.
// Blends the original scene with glow effects for use as lighting input.

@group(2) @binding(0)
var t_scene: texture_2d<f32>;

@group(2) @binding(1)
var s_scene: sampler;

@group(2) @binding(2)
var t_glow: texture_2d<f32>;

@group(2) @binding(3)
var s_glow: sampler;

@group(2) @binding(4)
var t_undarkened: texture_2d<f32>;

@group(2) @binding(5)
var s_undarkened: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(t_scene, s_scene, in.uv);
    let glow = textureSample(t_glow, s_glow, in.uv);
    let existing = textureSample(t_undarkened, s_undarkened, in.uv);

    let combined_rgb = mix(scene.rgb, glow.rgb, glow.a);
    let combined_a = max(scene.a, glow.a);
    let combined = vec4<f32>(combined_rgb, combined_a);

    let out_col = mix(existing, combined, combined.a);

    return vec4<f32>(clamp(out_col.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), combined.a);
}
