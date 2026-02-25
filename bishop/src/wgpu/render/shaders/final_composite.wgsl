// Final composite shader for spotlight compositing.
// Blends the scene composite with spotlight effects.

@group(2) @binding(0)
var t_scene_comp: texture_2d<f32>;

@group(2) @binding(1)
var s_scene_comp: sampler;

@group(2) @binding(2)
var t_spot: texture_2d<f32>;

@group(2) @binding(3)
var s_spot: sampler;

@group(2) @binding(4)
var t_final_comp: texture_2d<f32>;

@group(2) @binding(5)
var s_final_comp: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(t_scene_comp, s_scene_comp, in.uv);
    let spot = textureSample(t_spot, s_spot, in.uv);
    let existing = textureSample(t_final_comp, s_final_comp, in.uv);

    let current = mix(scene, scene + spot, spot.a);
    let out_col = mix(existing, current, current.a);

    return vec4<f32>(clamp(out_col.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
