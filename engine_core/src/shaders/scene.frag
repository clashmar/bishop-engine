#version 100
precision mediump float;
varying vec2 uv;

uniform sampler2D amb_tex;        
uniform sampler2D glow_tex;       
uniform sampler2D scene_comp_tex; 

void main() {
    vec4 amb  = texture2D(amb_tex, uv);   
    vec4 glow = texture2D(glow_tex, uv);  

    float src_a = amb.a;
    vec3 src_rgb = amb.rgb + glow.rgb;
    gl_FragColor = vec4(src_rgb, src_a);
}