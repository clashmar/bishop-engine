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