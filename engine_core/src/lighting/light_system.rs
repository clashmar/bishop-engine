// engine_core/src/lighting/light_system.rs
use macroquad::prelude::*;
use crate::{
    assets::asset_manager::AssetManager, 
    lighting::{
        glow::Glow, 
        light::Light, 
        light_shaders::*
    }
};

/// Max lights per layer.
pub const MAX_LIGHTS: usize = 10;

/// Helper struct that bundles the four cameras we need for a single layer
pub struct RenderCams {
    pub scene_cam: Camera2D,
    pub glow_cam: Camera2D,
    pub ambient_cam: Camera2D,
    pub spot_cam: Camera2D,
    pub composite_cam: Camera2D,
}

pub struct LightSystem {
    // Render targets
    pub scene_rt: RenderTarget,
    pub glow_rt: RenderTarget,
    pub ambient_rt: RenderTarget,
    pub spot_rt: RenderTarget,
    pub mask_rt: RenderTarget,
    pub composite_rt: RenderTarget,
    /// Materials
    pub glow_mat: Material,
    pub ambient_mat: Material,
    pub spot_mat: Material,
    pub composite_mat: Material,
    /// Cached light data
    pub pos: Vec<Vec2>,
    pub color: Vec<Vec3>,
    pub intensity: Vec<f32>,
    pub radius: Vec<f32>,
    pub spread: Vec<f32>,
    pub alpha: Vec<f32>,
    pub brightness: Vec<f32>,
    /// Cached glow data
    pub glow_brightness: Vec<f32>,
    pub glow_color: Vec<Vec3>,
    pub glow_color_int: Vec<f32>,
    pub glow_pos: Vec<Vec2>,
    pub glow_radius: Vec<f32>,
    pub glow_mask_width: Vec<f32>,
    pub glow_mask_height: Vec<f32>,
    pub glow_mask_pos: Vec<Vec2>,
    pub glow_mask_size: Vec<Vec2>,
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

        let glow_mat = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: GLOW_FRAGMENT_SHADER },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("GlowCount", UniformType::Int1),
                    UniformDesc::new("Brightness", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("Color", UniformType::Float3).array(MAX_LIGHTS),
                    UniformDesc::new("ColorIntensity", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("LightPos", UniformType::Float2).array(MAX_LIGHTS),
                    UniformDesc::new("Glow", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("maskWidth", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("maskHeight", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("maskPos", UniformType::Float2).array(MAX_LIGHTS),
                    UniformDesc::new("maskSize", UniformType::Float2).array(MAX_LIGHTS),
                    UniformDesc::new("screenWidth", UniformType::Float1),
                    UniformDesc::new("screenHeight", UniformType::Float1),
                    UniformDesc::new("Darkness", UniformType::Float1),
                ],
                textures: vec![
                    "scene_tex".to_string(),
                    "tex_mask0".to_string(),
                    "tex_mask1".to_string(),
                    "tex_mask2".to_string(),
                    "tex_mask3".to_string(),
                    "tex_mask4".to_string(),
                    "tex_mask5".to_string(),
                    "tex_mask6".to_string(),
                    "tex_mask7".to_string(),
                ],
                ..Default::default()
            },
        ).unwrap();

        let ambient_mat = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: AMB_FRAGMENT_SHADER },
            MaterialParams {
                uniforms: vec![UniformDesc::new("Darkness", UniformType::Float1)],
                textures: vec!["tex".to_string()],
                ..Default::default()
            },
        ).unwrap();

        let spot_mat = load_material(
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

        let composite_mat = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: COMPOSITE_FRAGMENT_SHADER },
            MaterialParams {
                textures: vec![
                    "ambient_tex".to_string(),
                    "spot_tex".to_string(),
                    "glow_tex".to_string(),
                    "composite_tex".to_string(),
                ],
                ..Default::default()
            },
        ).unwrap();

        Self {
            scene_rt: make_render_target(),
            glow_rt: make_render_target(),
            ambient_rt: make_render_target(),
            spot_rt: make_render_target(),
            mask_rt: make_render_target(),
            composite_rt: make_render_target(),
            glow_mat,
            ambient_mat,
            spot_mat,
            composite_mat,
            pos: vec![vec2(0.0, 0.0); MAX_LIGHTS],
            color: vec![vec3(0.0, 0.0, 0.0); MAX_LIGHTS],
            intensity: vec![0.0; MAX_LIGHTS],
            radius: vec![0.0; MAX_LIGHTS],
            spread: vec![0.0; MAX_LIGHTS],
            alpha: vec![0.0; MAX_LIGHTS],
            brightness: vec![0.0; MAX_LIGHTS],
            glow_brightness:vec![0.0; MAX_LIGHTS],
            glow_color: vec![vec3(0.0, 0.0, 0.0); MAX_LIGHTS],
            glow_color_int: vec![0.0; MAX_LIGHTS],
            glow_pos: vec![vec2(0.0, 0.0); MAX_LIGHTS],
            glow_radius: vec![0.0; MAX_LIGHTS],
            glow_mask_width: vec![0.0; MAX_LIGHTS],
            glow_mask_height: vec![0.0; MAX_LIGHTS],
            glow_mask_pos: vec![vec2(0.0, 0.0); MAX_LIGHTS],
            glow_mask_size: vec![vec2(0.0, 0.0); MAX_LIGHTS],
        }
    }

    /// Sets the mask render target background to white.
    pub fn init_mask_cam(&self) {
        let mask_cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(self.mask_rt.clone()),
            ..Default::default()
        };
        set_camera(&mask_cam);
        clear_background(WHITE);
        gl_use_default_material();
    }

    /// Sets, clears the given render target and returns the camera for it.
    pub fn clear_cam(rt: &RenderTarget) -> Camera2D {
        let cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom:   vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        set_camera(&cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        cam
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

    /// Reset the per‑frame glow buffers.
    pub fn clear_glow_buffers(&mut self) {
        for i in 0..MAX_LIGHTS {
            self.glow_brightness[i] = 0.0;
            self.glow_color[i] = vec3(0.0, 0.0, 0.0);
            self.glow_color_int[i] = 0.0;
            self.glow_pos[i] = vec2(0.0, 0.0);
            self.glow_radius[i] = 0.0;
            self.glow_mask_width[i] = 0.0;
            self.glow_mask_height[i] = 0.0;
            self.glow_mask_pos[i] = vec2(0.0, 0.0);
            self.glow_mask_size[i] = vec2(0.0, 0.0);
        }
    }

    pub fn run_glow_pass(
        &mut self,
        render_cam: &Camera2D,
        glows: Vec<(Vec2, &Glow)>,
        darkness: f32,
        asset_manager: &mut AssetManager,
    ) {
        self.glow_mat.set_texture("scene_tex", self.scene_rt.texture.clone());

        for (i, (world_pos, glow)) in glows.iter().take(MAX_LIGHTS).enumerate() {
            if let Some(id) = asset_manager.get_or_load(&glow.mask_sprite) {
                let screen_pos = render_cam.world_to_screen(*world_pos);
                self.glow_pos[i] = screen_pos;
                self.glow_color[i] = glow.color;
                self.glow_color_int[i] = glow.color_intensity;
                self.glow_brightness[i] = glow.brightness;
                self.glow_radius[i] = glow.glow_radius;
                self.glow_mask_pos[i] = glow.mask_pos;
                self.glow_mask_size[i] = glow.mask_size;

                // Texture dimensions
                if let Some((w, h)) = asset_manager.texture_size(id) {
                    self.glow_mask_width[i]  = w;
                    self.glow_mask_height[i] = h;
                }

                let tex = asset_manager.get_texture_from_id(id).clone();
                self.glow_mat.set_texture(&format!("tex_mask{}", i), tex);
            }
        }
        
        self.glow_mat.set_uniform("GlowCount", glows.len().min(MAX_LIGHTS) as i32);
        self.glow_mat.set_uniform_array("Brightness", &self.glow_brightness);
        self.glow_mat.set_uniform_array("Color", &self.glow_color);
        self.glow_mat.set_uniform_array("ColorIntensity", &self.glow_color_int);
        self.glow_mat.set_uniform_array("LightPos", &self.glow_pos);
        self.glow_mat.set_uniform_array("Glow", &self.glow_radius);
        self.glow_mat.set_uniform_array("maskWidth", &self.glow_mask_width);
        self.glow_mat.set_uniform_array("maskHeight", &self.glow_mask_height);
        self.glow_mat.set_uniform_array("maskPos", &self.glow_mask_pos);
        self.glow_mat.set_uniform_array("maskSize", &self.glow_mask_size);
        self.glow_mat.set_uniform("screenWidth", screen_width());
        self.glow_mat.set_uniform("screenHeight", screen_height());
        self.glow_mat.set_uniform("Darkness", darkness);
        
        LightSystem::clear_cam(&self.glow_rt);
        
        gl_use_material(&self.glow_mat);
        draw_texture_ex(
            &self.glow_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams::default(),
        );
        
        gl_use_default_material();
    }
    
    pub fn run_ambient_pass(&mut self, darkness: f32) {
        self.ambient_mat.set_texture("tex", self.scene_rt.texture.clone());
        self.ambient_mat.set_uniform("Darkness", darkness);

        LightSystem::clear_cam(&self.ambient_rt);

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
        render_cam: &Camera2D, 
        lights: Vec<(Vec2, Light)>,
        darkness: f32,
    ) {
        if !lights.is_empty() {
            let light_count = lights.len(); 

            for i in 0..light_count {
                let (pos, l) = &lights[i];
                let world_pos = *pos + l.pos;

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

            LightSystem::clear_cam(&self.spot_rt);

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
        self.composite_mat.set_texture("composite_tex", self.composite_rt.texture.clone());

        LightSystem::clear_cam(&self.composite_rt);

        gl_use_material(&self.composite_mat);
        draw_texture_ex(
            &self.composite_rt.texture,
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