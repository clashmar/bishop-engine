// Grid shader for editor grid overlay.
// Renders an infinite grid with anti-aliased lines that scale with zoom.
// VertexOutput is defined in vertex.wgsl which is concatenated with this shader.

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // UV (0..1) maps to the visible world area
    // visible_half_height = 1.0 / zoom (macroquad convention)
    // visible_half_width = aspect / zoom = (viewport.x / viewport.y) / zoom
    let aspect = params.viewport_size.x / params.viewport_size.y;
    let visible_half_height = 1.0 / params.camera_zoom;
    let visible_half_width = aspect / params.camera_zoom;

    // Convert UV to world position
    let world_offset = vec2<f32>(
        (in.uv.x - 0.5) * 2.0 * visible_half_width,
        (in.uv.y - 0.5) * 2.0 * visible_half_height
    );
    let world_pos = world_offset + params.camera_pos;

    // Calculate distance to nearest grid line
    let grid_coord = world_pos / params.grid_size;
    let frac_coord = fract(grid_coord + 0.5) - 0.5;
    let dist_to_line = abs(frac_coord) * params.grid_size;

    // Find minimum distance to either vertical or horizontal line
    let min_dist = min(dist_to_line.x, dist_to_line.y);

    // Convert line thickness to world units
    // line_thickness is a scale factor, dividing by zoom gives world thickness
    let world_thickness = params.line_thickness / params.camera_zoom;

    // Anti-aliased line rendering
    let half_thickness = world_thickness * 0.5;
    let alpha = 1.0 - smoothstep(0.0, half_thickness, min_dist);

    return vec4<f32>(params.line_color.rgb, params.line_color.a * alpha);
}
