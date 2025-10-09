// engine_core/src/lighting/light_system.rs
use crate::{
    ecs::world_ecs::WorldEcs, 
    lighting::{light::Light, lighting_stage::{Stage, Uniforms}}
};
use macroquad::{miniquad::PassAction, prelude::*};

pub const MAX_LIGHTS: usize = 8;

pub fn render_lighting(
    world_ecs: &WorldEcs,
    camera: &Camera2D,
    scene_tex: &Texture2D,
    target_pass: miniquad::RenderPass,
    lighting_stage: &mut Stage,
) {
    let mut position_px = [(0.0_f32, 0.0_f32); MAX_LIGHTS];
    let mut radius_px = [(0.0_f32); MAX_LIGHTS];
    let mut intensity = [(0.0_f32); MAX_LIGHTS];
    let mut colour = [(0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32); MAX_LIGHTS];
    let mut count = 0usize;

    for (_, light) in world_ecs.get_store::<Light>().data.iter() {
        if count >= MAX_LIGHTS {
            break;
        }

        let screen = camera.world_to_screen(light.position);
        position_px[count] = (screen.x, screen.y);

        let screen_radius = light.radius / camera.zoom.x;
        radius_px[count] = screen_radius;

        intensity[count] = light.intensity;
        colour[count] = (light.colour.x, light.colour.y, light.colour.z, 1.0);
        count += 1;
    }

    let gl = unsafe { get_internal_gl() };
    let uniforms = Uniforms {
        light_count: count as i32,
        light_position: position_px,
        light_radius: radius_px,
        light_intensity: intensity,
        light_colour: colour,
        scene_size: (scene_tex.width(), scene_tex.height()),
    };
    
    gl.quad_context.apply_uniforms(miniquad::UniformsSource::table(&uniforms));

    // Bind the scene texture
    lighting_stage.bindings.images = vec![scene_tex.raw_miniquad_id()];

    // Draw fullscreen quad
    gl.quad_context.apply_pipeline(&lighting_stage.pipeline);
    gl.quad_context.begin_pass(Some(target_pass), PassAction::Nothing);
    gl.quad_context.apply_bindings(&lighting_stage.bindings);
    gl.quad_context.draw(0, 6, 1);
    gl.quad_context.end_render_pass();
}