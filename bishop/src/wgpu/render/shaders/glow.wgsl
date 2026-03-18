// Glow shader for multi-source glow with blur.
// Renders glow effects from multiple mask textures with emission-based blur.

const MAX_LIGHTS: u32 = 10u;

struct GlowData {
    mask_pos: vec2<f32>,
    mask_size: vec2<f32>,
    color: vec3<f32>,
    brightness: f32,
    intensity: f32,
    emission: f32,
    _pad: vec2<f32>,
}

struct GlowUniforms {
    screen_size: vec2<f32>,
    glow_count: i32,
    _pad: f32,
    glows: array<GlowData, 10>,
}

@group(1) @binding(0)
var<uniform> params: GlowUniforms;

@group(2) @binding(0)
var t_scene: texture_2d<f32>;

@group(2) @binding(1)
var s_scene: sampler;

// Individual mask textures (using if-chain fallback for compatibility)
@group(3) @binding(0)
var t_mask0: texture_2d<f32>;

@group(3) @binding(1)
var s_mask: sampler;

@group(3) @binding(2)
var t_mask1: texture_2d<f32>;

@group(3) @binding(3)
var t_mask2: texture_2d<f32>;

@group(3) @binding(4)
var t_mask3: texture_2d<f32>;

@group(3) @binding(5)
var t_mask4: texture_2d<f32>;

@group(3) @binding(6)
var t_mask5: texture_2d<f32>;

@group(3) @binding(7)
var t_mask6: texture_2d<f32>;

@group(3) @binding(8)
var t_mask7: texture_2d<f32>;

@group(3) @binding(9)
var t_mask8: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn sample_mask(index: u32, uv_mask: vec2<f32>) -> f32 {
    // Reject coordinates outside the mask
    if uv_mask.x < 0.0 || uv_mask.x > 1.0 || uv_mask.y < 0.0 || uv_mask.y > 1.0 {
        return 0.0;
    }

    if index == 0u {
        return textureSample(t_mask0, s_mask, uv_mask).a;
    } else if index == 1u {
        return textureSample(t_mask1, s_mask, uv_mask).a;
    } else if index == 2u {
        return textureSample(t_mask2, s_mask, uv_mask).a;
    } else if index == 3u {
        return textureSample(t_mask3, s_mask, uv_mask).a;
    } else if index == 4u {
        return textureSample(t_mask4, s_mask, uv_mask).a;
    } else if index == 5u {
        return textureSample(t_mask5, s_mask, uv_mask).a;
    } else if index == 6u {
        return textureSample(t_mask6, s_mask, uv_mask).a;
    } else if index == 7u {
        return textureSample(t_mask7, s_mask, uv_mask).a;
    } else if index == 8u {
        return textureSample(t_mask8, s_mask, uv_mask).a;
    }
    return 0.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_scene = textureSample(t_scene, s_scene, in.uv).rgb;

    let frag_screen = in.uv * params.screen_size;
    var final_mask: f32 = 0.0;
    var glow_accum = vec3<f32>(0.0);

    let glow_count = u32(params.glow_count);
    for (var i = 0u; i < glow_count; i = i + 1u) {
        if i >= MAX_LIGHTS {
            break;
        }

        let glow = params.glows[i];
        let rel = (frag_screen - glow.mask_pos) / glow.mask_size;
        let c00 = sample_mask(i, rel);

        // 3x3 blur
        let pixel_size = 1.0 / glow.mask_size;
        var sum: f32 = 0.0;
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(-glow.emission, -glow.emission));
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(0.0, -glow.emission));
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(glow.emission, -glow.emission));
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(glow.emission, 0.0));
        sum = sum + c00;
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(glow.emission, glow.emission));
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(0.0, glow.emission));
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(-glow.emission, glow.emission));
        sum = sum + sample_mask(i, rel + pixel_size * vec2<f32>(-glow.emission, 0.0));
        let avg = sum / 9.0;

        let blurred = mix(c00, avg, clamp(glow.emission, 0.0, 1.0));
        final_mask = max(final_mask, blurred);

        let glow_color = glow.color * glow.brightness * blurred;

        let tinted = mix(base_scene, glow.color, glow.intensity * blurred);
        let tint_contribution = (tinted - base_scene) * blurred;

        glow_accum = glow_accum + glow_color + tint_contribution;
    }

    let out_rgb = glow_accum;
    let out_a = final_mask;

    return vec4<f32>(clamp(out_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), out_a);
}
