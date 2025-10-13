// engine_core/src/lighting/light_shaders.rs

pub const VERTEX_SHADER: &str = r#"
#version 100
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

pub const AMB_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D tex;
uniform float Darkness;

void main() {
    vec4 base = texture2D(tex, uv);
    vec3 scene = base.rgb;

    vec3 darkened = mix(scene, vec3(0.0), Darkness);

    gl_FragColor = vec4(clamp(darkened, 0.0, 1.0), base.a);
}
"#;

pub const SPOT_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D tex;
uniform sampler2D light_mask;

#define MAX_LIGHTS 10

uniform int LightCount; 
uniform vec2 LightPos[MAX_LIGHTS];
uniform vec3 LightColor[MAX_LIGHTS];
uniform float LightIntensity[MAX_LIGHTS];
uniform float LightRadius[MAX_LIGHTS];
uniform float LightSpread[MAX_LIGHTS];
uniform float LightAlpha[MAX_LIGHTS];
uniform float LightBrightness[MAX_LIGHTS];
uniform float ScreenWidth;
uniform float ScreenHeight;
uniform float Darkness;

void main() {
    vec4 base = texture2D(tex, uv);
    vec3 scene = base.rgb;

    float maskVal = texture2D(light_mask, uv).r;
    if (maskVal < 0.01) {
        gl_FragColor = vec4(scene, 0.0);
        return;
    }

    vec2 fragPos = uv * vec2(ScreenWidth, ScreenHeight);
    vec3 result = vec3(0.0);
    float totalMask = 0.0;

    for (int i = 0; i < LightCount; ++i) {
        if (i >= MAX_LIGHTS) break;

        float dist = distance(fragPos, LightPos[i]);
        float mask = 1.0 - smoothstep(LightRadius[i],
                                      LightRadius[i] + LightSpread[i],
                                      dist);
        mask *= LightAlpha[i];

        // colour tint
        vec3 tinted = mix(scene, LightColor[i], LightIntensity[i]);
        // final lit colour for this light
        vec3 lit = mix(scene, tinted, mask);
        lit += LightBrightness[i] * LightColor[i] * mask;

        // contribution (same formula you already used)
        vec3 contrib = (lit - scene * (1.0 - Darkness)) * mask;

        result += contrib;
        totalMask += mask;
    }

    gl_FragColor = vec4(clamp(result, 0.0, 1.0), totalMask);
}
"#;

pub const GLOW_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

#define MAX_LIGHTS 10

varying vec2 uv;

uniform sampler2D scene_tex;

uniform sampler2D tex_mask0;
uniform sampler2D tex_mask1;
uniform sampler2D tex_mask2;
uniform sampler2D tex_mask3;
uniform sampler2D tex_mask4;
uniform sampler2D tex_mask5;
uniform sampler2D tex_mask6;
uniform sampler2D tex_mask7;

uniform int GlowCount;
uniform float Brightness[MAX_LIGHTS];
uniform float Intensity[MAX_LIGHTS];
uniform vec3 Color[MAX_LIGHTS];
uniform float Glow[MAX_LIGHTS];
uniform float maskWidth[MAX_LIGHTS];
uniform float maskHeight[MAX_LIGHTS];
uniform vec2 maskPos[MAX_LIGHTS];
uniform vec2 maskSize[MAX_LIGHTS];
uniform float screenWidth;
uniform float screenHeight;
uniform float Darkness; 

// Helper function to sample the correct mask
float sampleMask(int i, vec2 uvMask) {
    if (uvMask.x < 0.0 || uvMask.x > 1.0 || uvMask.y < 0.0 || uvMask.y > 1.0)
        return 0.0;

    if (i == 0) return texture2D(tex_mask0, uvMask).a;
    if (i == 1) return texture2D(tex_mask1, uvMask).a;
    if (i == 2) return texture2D(tex_mask2, uvMask).a;
    if (i == 3) return texture2D(tex_mask3, uvMask).a;
    if (i == 4) return texture2D(tex_mask4, uvMask).a;
    if (i == 5) return texture2D(tex_mask5, uvMask).a;
    if (i == 6) return texture2D(tex_mask6, uvMask).a;
    if (i == 7) return texture2D(tex_mask7, uvMask).a;
    return 0.0;
}

void main() {
    vec4 base = texture2D(scene_tex, uv);
    vec3 scene = base.rgb;

    vec2 fragScreen = uv * vec2(screenWidth, screenHeight);

    vec3 contribution = vec3(0.0);
    float finalMask = 0.0;

    for (int i = 0; i < GlowCount; ++i) {
        if (i >= MAX_LIGHTS) break;

        // Map fragment to mask UV
        vec2 rel = (fragScreen - (maskPos[i] - maskSize[i] * 0.5)) / maskSize[i];

        float c00 = sampleMask(i, rel);

        vec2 pixelSize = vec2(1.0 / maskWidth[i], 1.0 / maskHeight[i]);

        float sum = 0.0;
        sum += sampleMask(i, rel + pixelSize * vec2(-Glow[i], -Glow[i]));
        sum += sampleMask(i, rel + pixelSize * vec2(0.0, -Glow[i]));
        sum += sampleMask(i, rel + pixelSize * vec2(Glow[i], -Glow[i]));
        sum += sampleMask(i, rel + pixelSize * vec2(Glow[i], 0.0));
        sum += c00;
        sum += sampleMask(i, rel + pixelSize * vec2(Glow[i], Glow[i]));
        sum += sampleMask(i, rel + pixelSize * vec2(0.0, Glow[i]));
        sum += sampleMask(i, rel + pixelSize * vec2(-Glow[i], Glow[i]));
        sum += sampleMask(i, rel + pixelSize * vec2(-Glow[i], 0.0));
        float avg = sum / 9.0;

        float s = clamp(Glow[i], 0.0, 1.0);
        float blurred = mix(c00, avg, s);

        finalMask = max(finalMask, blurred);

        // Apply lighting
        vec3 tinted = mix(scene, Color[i], Intensity[i]);
        vec3 lit = mix(scene, tinted, blurred);
        lit += Brightness[i] * Color[i] * blurred;

        contribution += (lit - scene * (1.0 - Darkness)) * blurred;
    }

    gl_FragColor = vec4(clamp(contribution, 0.0, 1.0), finalMask);
}
"#;

pub const COMPOSITE_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D ambient_tex;
uniform sampler2D spot_tex;
uniform sampler2D glow_tex;
uniform sampler2D composite_tex;

void main() {
    vec4 existing = texture2D(composite_tex, uv);
    vec4 ambient = texture2D(ambient_tex, uv);
    vec4 spot = texture2D(spot_tex, uv);

    // Combine ambient and spotlight pass for this layer
    vec4 current = mix(ambient, ambient + spot, spot.a);

    // Blend current layer over existing composite
    vec4 outCol = mix(existing, current, current.a);

    gl_FragColor = vec4(clamp(outCol.rgb, 0.0, 1.0), 1.0);
}
"#;