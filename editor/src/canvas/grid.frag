// editor/src/canvas/grid.frag
#version 100
precision mediump float;

varying vec2 uv;

uniform vec2 camera_pos;
uniform float camera_zoom;
uniform vec2 viewport_size;
uniform float grid_size;
uniform vec4 line_color;
uniform float line_thickness;

void main() {
    // Convert UV (0..1) to screen coordinates centered at (0,0)
    vec2 screen_pos = (uv - 0.5) * viewport_size;

    // Convert screen position to world position
    vec2 world_pos = screen_pos / camera_zoom + camera_pos;

    // Calculate distance to nearest grid line
    vec2 grid_coord = world_pos / grid_size;
    vec2 frac_coord = fract(grid_coord + 0.5) - 0.5;
    vec2 dist_to_line = abs(frac_coord) * grid_size;

    // Find minimum distance to either vertical or horizontal line
    float min_dist = min(dist_to_line.x, dist_to_line.y);

    // Convert thickness to world units based on zoom
    // Higher zoom = smaller world units visible = thinner lines in world space
    float world_thickness = line_thickness / camera_zoom;

    // Anti-aliased line rendering
    float half_thickness = world_thickness * 0.5;
    float alpha = 1.0 - smoothstep(0.0, half_thickness, min_dist);

    gl_FragColor = vec4(line_color.rgb, line_color.a * alpha);
}
