// engine_core/src/lighting/light_system.rs
use macroquad::prelude::*;
use crate::{ecs::component::Position, lighting::{light::Light, light_shaders::*}};

/// Max lights per layer.
pub const MAX_LIGHTS: usize = 10;

/// Helper struct that bundles the four cameras we need for a single layer
pub struct RenderCams {
    pub scene_cam: Camera2D,
    pub ambient_cam: Camera2D,
    pub spot_cam: Camera2D,
    pub glow_cam: Camera2D,
    pub mask_cam: Camera2D,
    pub depth_cam: Camera2D,
}

pub struct LightSystem {
    // Render targets
    pub scene_rt: RenderTarget,
    pub ambient_rt: RenderTarget,
    pub spot_rt: RenderTarget,
    pub glow_rt: RenderTarget,
    pub mask_rt: RenderTarget,
    pub depth_mask_rt: RenderTarget,
    /// Materials
    pub ambient_mat: Material,
    pub spot_mat: Material,
    pub glow_mat: Material,
    pub composite_mat: Material,
    /// Cached light data
    pub pos: Vec<Vec2>,
    pub color: Vec<Vec3>,
    pub intensity: Vec<f32>,
    pub radius: Vec<f32>,
    pub spread: Vec<f32>,
    pub alpha: Vec<f32>,
    pub brightness: Vec<f32>,
}

impl LightSystem {
    pub fn new() -> Self {
        // Render‑targets are created with the screen size.
        let width = screen_width() as u32;
        let height = screen_height() as u32;

        let make_render_target = || {
            let rt = render_target(width, height);
            rt.texture.set_filter(FilterMode::Nearest);
            rt
        };

        // Load the four shaders once (they are the same for every layer)
        let ambient_material = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: AMB_FRAGMENT_SHADER },
            MaterialParams {
                uniforms: vec![UniformDesc::new("Darkness", UniformType::Float1)],
                textures: vec!["tex".to_string()],
                ..Default::default()
            },
        ).unwrap();

        let spot_material = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: SPOT_FRAGMENT_SHADER },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("LightCount", UniformType::Int1),
                    UniformDesc::new("LightPos", UniformType::Float2).array(MAX_LIGHTS),
                    UniformDesc::new("LightColor", UniformType::Float3).array(MAX_LIGHTS),
                    UniformDesc::new("LightIntensity", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("LightRadius", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("LightSpread", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("LightAlpha", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("LightBrightness", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("ScreenWidth", UniformType::Float1),
                    UniformDesc::new("ScreenHeight", UniformType::Float1),
                    UniformDesc::new("Darkness", UniformType::Float1),
                ],
                textures: vec!["tex".to_string(), "light_mask".to_string()],
                ..Default::default()
            },
        ).unwrap();

        let glow_material = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: GLOW_FRAGMENT_SHADER },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("Brightness", UniformType::Float1),
                    UniformDesc::new("Color", UniformType::Float3),
                    UniformDesc::new("ColorIntensity", UniformType::Float1),
                    UniformDesc::new("LightPos", UniformType::Float2),
                    UniformDesc::new("Glow", UniformType::Float1),
                    UniformDesc::new("maskWidth", UniformType::Float1),
                    UniformDesc::new("maskHeight", UniformType::Float1),
                    UniformDesc::new("maskPos", UniformType::Float2),
                    UniformDesc::new("maskSize", UniformType::Float2),
                    UniformDesc::new("screenWidth", UniformType::Float1),
                    UniformDesc::new("screenHeight", UniformType::Float1),
                    UniformDesc::new("Darkness", UniformType::Float1),
                ],
                textures: vec!["scene_tex".to_string(), "tex_mask".to_string()],
                ..Default::default()
            },
        ).unwrap();

        let composite_material = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: COMPOSITE_FRAGMENT_SHADER },
            MaterialParams {
                textures: vec![
                    "ambient_tex".to_string(),
                    "spot_tex".to_string(),
                    "glow_tex".to_string(),
                ],
                ..Default::default()
            },
        ).unwrap();

        Self {
            scene_rt: make_render_target(),
            ambient_rt: make_render_target(),
            spot_rt: make_render_target(),
            glow_rt: make_render_target(),
            mask_rt: make_render_target(),
            depth_mask_rt: make_render_target(),
            ambient_mat: ambient_material,
            spot_mat: spot_material,
            glow_mat: glow_material,
            composite_mat: composite_material,
            pos: vec![vec2(0.0, 0.0); MAX_LIGHTS],
            color: vec![vec3(0.0, 0.0, 0.0); MAX_LIGHTS],
            intensity: vec![0.0; MAX_LIGHTS],
            radius: vec![0.0; MAX_LIGHTS],
            spread: vec![0.0; MAX_LIGHTS],
            alpha: vec![0.0; MAX_LIGHTS],
            brightness: vec![0.0; MAX_LIGHTS],
        }
    }

    pub fn render_cams(&self, render_cam: &Camera2D) -> RenderCams {
        // Scene cam has different requirements
        let scene_cam = Camera2D {
            target: render_cam.target,
            zoom: render_cam.zoom,
            render_target: Some(self.scene_rt.clone()),
            ..Default::default()
        };
        set_camera(&scene_cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));

        // Depth cam should not be cleared each layer
        let depth_cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(self.depth_mask_rt.clone()),
            ..Default::default()
        };
        set_camera(&scene_cam);

        // Helper to create the other cameras and clear textures
        let clear_rt = |rt: &RenderTarget| {
            let cam = Camera2D {
                target: vec2(screen_width() * 0.5, screen_height() * 0.5),
                zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
                render_target: Some(rt.clone()),
                ..Default::default()
            };
            set_camera(&cam);
            clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
            cam
        };
        
        let ambient_cam = clear_rt(&self.ambient_rt);
        let spot_cam = clear_rt(&self.spot_rt);
        let glow_cam = clear_rt(&self.glow_rt);
        let mask_cam = clear_rt(&self.mask_rt);

        // Build the four cameras that will be used for drawing.
        RenderCams {
            scene_cam,
            ambient_cam,
            spot_cam,
            glow_cam,
            mask_cam,
            depth_cam,
        }
    }

    /// Reset the per‑frame light buffers.
    pub fn clear_light_buffers(&mut self) {
        for i in 0..MAX_LIGHTS {
            self.pos[i] = vec2(0.0, 0.0);
            self.color[i] = vec3(0.0, 0.0, 0.0);
            self.intensity[i] = 0.0;
            self.radius[i] = 0.0;
            self.spread[i] = 0.0;
            self.alpha[i] = 0.0;
            self.brightness[i] = 0.0;
        }
    }
    
    pub fn run_ambient_pass(&mut self, camera: &Camera2D, darkness: f32) {
        self.ambient_mat.set_texture("tex", self.scene_rt.texture.clone());
        self.ambient_mat.set_uniform("Darkness", darkness);

        set_camera(camera);

        gl_use_material(&self.ambient_mat);
        draw_texture_ex(
            &self.ambient_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    pub fn run_spotlight_pass(
        &mut self, 
        spot_cam: &Camera2D, 
        render_cam: &Camera2D, 
        lights: Vec<(Position, &Light)>,
        darkness: f32,
    ) {
        if !lights.is_empty() {
            let light_count = lights.len(); 

            for i in 0..light_count {
                let (entity_pos, l) = &lights[i];
                let world_pos = entity_pos.position + l.pos;

                self.pos[i] = render_cam.world_to_screen(world_pos);
                self.radius[i] = world_distance_to_screen(render_cam, l.radius);
                self.spread[i] = world_distance_to_screen(render_cam, l.spread);
                self.color[i] = l.color;
                self.intensity[i] = l.intensity;
                self.alpha[i] = l.alpha;
                self.brightness[i] = l.brightness;
            }

            self.spot_mat.set_texture("tex", self.scene_rt.texture.clone());
            self.spot_mat.set_texture("light_mask", self.mask_rt.texture.clone());

            self.spot_mat.set_uniform("LightCount", light_count as i32);
            self.spot_mat.set_uniform_array("LightPos", &self.pos);
            self.spot_mat.set_uniform_array("LightColor", &self.color);
            self.spot_mat.set_uniform_array("LightIntensity", &self.intensity);
            self.spot_mat.set_uniform_array("LightRadius", &self.radius);
            self.spot_mat.set_uniform_array("LightSpread", &self.spread);
            self.spot_mat.set_uniform_array("LightAlpha", &self.alpha);
            self.spot_mat.set_uniform_array("LightBrightness", &self.brightness);
            self.spot_mat.set_uniform("ScreenWidth", screen_width());
            self.spot_mat.set_uniform("ScreenHeight", screen_height());
            self.spot_mat.set_uniform("Darkness", darkness);

            set_camera(spot_cam);

            gl_use_material(&self.spot_mat);
            draw_texture_ex(
                &self.spot_rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    ..Default::default()
                },
            );
            gl_use_default_material();
        }
    }

    pub fn run_composite_pass(&mut self) {
        self.composite_mat.set_texture("ambient_tex", self.ambient_rt.texture.clone());
        self.composite_mat.set_texture("spot_tex", self.spot_rt.texture.clone());
        self.composite_mat.set_texture("glow_tex", self.glow_rt.texture.clone());

        // Draw everything
        set_default_camera();
        gl_use_material(&self.composite_mat);
        draw_texture_ex(
            &self.scene_rt.texture, // any of the three works for size
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}

pub fn world_distance_to_screen(cam: &Camera2D, distance: f32) -> f32 {
    let scale = cam.zoom.x * screen_width() * 0.5; 
    (distance * scale).abs()
}