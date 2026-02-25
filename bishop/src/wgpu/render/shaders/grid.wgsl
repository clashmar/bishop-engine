// Grid shader for editor grid overlay.
// Renders an infinite grid with anti-aliased lines that scale with zoom.

struct GridUniforms {
    camera_pos: vec2<f32>,
    camera_zoom: f32,
    grid_size: f32,
    viewport_size: vec2<f32>,
    line_thickness: f32,
    _pad: f32,
    line_color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> params: GridUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV (0..1) to screen coordinates centered at (0,0)
    let screen_pos = (in.uv - 0.5) * params.viewport_size;

    // Convert screen position to world position
    let world_pos = screen_pos / params.camera_zoom + params.camera_pos;

    // Calculate distance to nearest grid line
    let grid_coord = world_pos / params.grid_size;
    let frac_coord = fract(grid_coord + 0.5) - 0.5;
    let dist_to_line = abs(frac_coord) * params.grid_size;

    // Find minimum distance to either vertical or horizontal line
    let min_dist = min(dist_to_line.x, dist_to_line.y);

    // Convert thickness to world units based on zoom
    // Higher zoom = smaller world units visible = thinner lines in world space
    let world_thickness = params.line_thickness / params.camera_zoom;

    // Anti-aliased line rendering
    let half_thickness = world_thickness * 0.5;
    let alpha = 1.0 - smoothstep(0.0, half_thickness, min_dist);

    return vec4<f32>(params.line_color.rgb, params.line_color.a * alpha);
}
