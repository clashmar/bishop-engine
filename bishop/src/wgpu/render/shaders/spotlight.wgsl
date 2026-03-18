// Spotlight shader for dynamic point lights.
// Renders multiple spotlights with smooth falloff and color tinting.

const MAX_LIGHTS: u32 = 10u;

struct SpotLightData {
    pos: vec2<f32>,
    intensity: f32,
    radius: f32,
    color: vec3<f32>,
    spread: f32,
    alpha: f32,
    brightness: f32,
    _pad: vec2<f32>,
}

struct SpotUniforms {
    screen_size: vec2<f32>,
    darkness: f32,
    light_count: i32,
    lights: array<SpotLightData, 10>,
}

@group(1) @binding(0)
var<uniform> params: SpotUniforms;

@group(2) @binding(0)
var t_scene: texture_2d<f32>;

@group(2) @binding(1)
var s_scene: sampler;

@group(2) @binding(2)
var t_light_mask: texture_2d<f32>;

@group(2) @binding(3)
var s_light_mask: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(t_scene, s_scene, in.uv);
    let scene = base.rgb;

    let mask_val = textureSample(t_light_mask, s_light_mask, in.uv).r;
    if mask_val < 0.01 {
        return vec4<f32>(scene, 0.0);
    }

    let frag_pos = in.uv * params.screen_size;
    var result = vec3<f32>(0.0);
    var total_mask: f32 = 0.0;

    let light_count = u32(params.light_count);
    for (var i = 0u; i < light_count; i = i + 1u) {
        if i >= MAX_LIGHTS {
            break;
        }

        let light = params.lights[i];
        let dist = distance(frag_pos, light.pos);
        var mask = 1.0 - smoothstep(light.radius, light.radius + light.spread, dist);
        mask = mask * light.alpha;

        let tinted = mix(scene, light.color, light.intensity);
        let lit = mix(scene, tinted, mask);
        let bright = lit + light.brightness * light.color * mask;

        let contrib = (bright - scene * (1.0 - params.darkness)) * mask;

        result = result + contrib;
        total_mask = total_mask + mask;
    }

    let normalized_mask = clamp(total_mask, 0.0, 1.0);
    return vec4<f32>(clamp(scene + result, vec3<f32>(0.0), vec3<f32>(1.0)), normalized_mask);
}
