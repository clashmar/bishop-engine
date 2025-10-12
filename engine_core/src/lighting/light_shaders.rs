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
    vec4  base  = texture2D(tex, uv);
    vec3  scene = base.rgb;

    float maskVal = texture2D(light_mask, uv).r;
    if (maskVal < 0.01) {
        gl_FragColor = vec4(scene, 0.0);
        return;
    }

    vec2  fragPos = uv * vec2(ScreenWidth, ScreenHeight);
    vec3  result = vec3(0.0);
    float totalMask = 0.0;

    for (int i = 0; i < LightCount; ++i) {
        if (i >= MAX_LIGHTS) break;

        float dist = distance(fragPos, LightPos[i]);
        float mask = 1.0 - smoothstep(LightRadius[i],
                                      LightRadius[i] + LightSpread[i],
                                      dist);
        mask *= LightAlpha[i];                     // SpotAlpha

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

varying vec2 uv;

uniform sampler2D scene_tex;
uniform sampler2D tex_mask;

uniform float Brightness;
uniform vec3 Color;
uniform float ColorIntensity;
uniform vec2 LightPos;
uniform float Glow;
uniform float maskWidth;
uniform float maskHeight;
uniform vec2 maskPos;
uniform vec2 maskSize;
uniform float screenWidth;
uniform float screenHeight;
uniform float Darkness; 

float sampleMask(vec2 uvMask)
{
    if (uvMask.x < 0.0 || uvMask.x > 1.0 ||
        uvMask.y < 0.0 || uvMask.y > 1.0) {
        return 0.0;
    }
    return texture2D(tex_mask, uvMask).a;
}

void main() {
    vec4 base = texture2D(scene_tex, uv);
    vec3 scene = base.rgb;
    float finalMask = 1.0;

    vec2 fragScreen = uv * vec2(screenWidth, screenHeight);

    vec2 rel = (fragScreen - (maskPos - maskSize * 0.5)) / maskSize;

    float c00 = sampleMask(rel);

    vec2 pixelSize = vec2(1.0 / maskWidth, 1.0 / maskHeight);

    float sum = 0.0;

    sum += sampleMask(rel + pixelSize * vec2(-Glow, -Glow));
    sum += sampleMask(rel + pixelSize * vec2( 0.0,  -Glow));
    sum += sampleMask(rel + pixelSize * vec2( Glow, -Glow));
    sum += sampleMask(rel + pixelSize * vec2( Glow,  0.0));
    sum += c00; // center
    sum += sampleMask(rel + pixelSize * vec2( Glow,  Glow));
    sum += sampleMask(rel + pixelSize * vec2( 0.0,  Glow));
    sum += sampleMask(rel + pixelSize * vec2(-Glow,  Glow));
    sum += sampleMask(rel + pixelSize * vec2(-Glow,  0.0));
    float avg = sum / 9.0;

    float s = clamp(Glow, 0.0, 1.0);
    float blurred = mix(c00, avg, s);

    finalMask = max(c00, blurred);

    // Apply lighting
    vec3 tinted = mix(scene, Color, ColorIntensity);
    vec3 lit = mix(scene, tinted, finalMask);
    lit += Brightness * Color * finalMask;

    vec3 contribution = (lit - scene * (1.0 - Darkness)) * finalMask;

    gl_FragColor = vec4(contribution, finalMask);
}
"#;

pub const COMPOSITE_FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D ambient_tex;
uniform sampler2D spot_tex;
uniform sampler2D glow_tex;

void main() {
    vec4 ambient = texture2D(ambient_tex, uv);
    vec4 spot = texture2D(spot_tex, uv);
    vec4 glow = texture2D(glow_tex, uv);

    gl_FragColor = clamp(ambient + spot + glow, 0.0, 1.0);
}
"#;