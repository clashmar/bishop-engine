// engine_core/src/shaders/scene.frag
#version 100
precision mediump float;
varying vec2 uv;

uniform sampler2D amb_tex;
uniform sampler2D glow_tex; 
uniform sampler2D scene_comp_tex;

void main() {
    vec4 amb   = texture2D(amb_tex, uv);
    vec4 glow  = texture2D(glow_tex, uv);
    vec4 prev  = texture2D(scene_comp_tex, uv);

    vec4 src = amb + glow;

    vec3 out_rgb = src.rgb * src.a + prev.rgb * (1.0 - src.a);
    float out_a  = src.a   + prev.a   * (1.0 - src.a);

    gl_FragColor = vec4(clamp(out_rgb, 0.0, 1.0), clamp(out_a, 0.0, 1.0));
}