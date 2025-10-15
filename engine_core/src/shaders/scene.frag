// engine_core/src/shaders/scene.frag
#version 100
precision mediump float;

varying vec2 uv;

uniform sampler2D amb_tex;        
uniform sampler2D glow_tex;       
uniform sampler2D scene_comp_tex; 

void main() {
    vec4 amb = texture2D(amb_tex, uv);
    vec4 glow = texture2D(glow_tex, uv); 
    vec4 existing = texture2D(scene_comp_tex, uv);

    vec3 combinedRGB = mix(amb.rgb, glow.rgb, glow.a);
    float combinedA = max(amb.a, glow.a); 

    vec4 combined = vec4(combinedRGB, combinedA);

    vec4 outCol = mix(existing, combined, combined.a);

    gl_FragColor = vec4(clamp(outCol.rgb,0.0,1.0), combined.a);
}