// engine_core/src/lighting/light_system.rs
use macroquad::{miniquad::{BlendFactor, BlendState, BlendValue, Equation}, prelude::*};
use crate::{
    assets::asset_manager::AssetManager, 
    lighting::{
        glow::Glow, 
        light::Light, 
    }, 
    shaders::shaders::*
};

/// Max lights per layer.
pub const MAX_LIGHTS: usize = 10;

#[derive(Clone, Copy, Default)]
pub struct LightBuffer {
    pos: Vec2,
    color: Vec3,
    intensity: f32,
    radius: f32,
    spread: f32,
    alpha: f32,
    brightness: f32,
}

#[derive(Clone, Copy, Default)]
pub struct GlowBuffer {
    brightness: f32,
    color: Vec3,
    intensity: f32,
    pos: Vec2,
    emission: f32,
    mask_size: Vec2,
}

pub struct LightSystem {
    // Render targets
    pub scene_rt: RenderTarget,
    pub ambient_rt: RenderTarget,
    pub glow_rt: RenderTarget,
    pub undarkened_rt: RenderTarget,
    pub spot_rt: RenderTarget,
    pub mask_rt: RenderTarget,
    pub scene_comp_rt: RenderTarget,
    pub final_comp_rt: RenderTarget,
    /// Materials
    pub ambient_mat: Material,
    pub glow_mat: Material,
    pub undarkened_mat: Material, 
    pub spot_mat: Material,
    pub scene_comp_mat: Material,
    pub final_comp_mat: Material,
    /// Cached light data
    light_bufffers: [LightBuffer; MAX_LIGHTS],
    glow_bufffers: [GlowBuffer; MAX_LIGHTS],
}

impl LightSystem {
    pub fn new() -> Self {
        // Render targets are created with the screen size
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
                    UniformDesc::new("Intensity", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("Color", UniformType::Float3).array(MAX_LIGHTS),
                    UniformDesc::new("Emission", UniformType::Float1).array(MAX_LIGHTS),
                    UniformDesc::new("maskPos", UniformType::Float2).array(MAX_LIGHTS),
                    UniformDesc::new("maskSize", UniformType::Float2).array(MAX_LIGHTS),
                    UniformDesc::new("screenWidth", UniformType::Float1),
                    UniformDesc::new("screenHeight", UniformType::Float1),
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
                    "tex_mask8".to_string(),
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

        let undarkened_mat = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: UNDARKENED_FRAGMENT_SHADER,
            },
            MaterialParams {
                textures: vec![
                    "scene_tex".to_string(),
                    "glow_tex".to_string(),
                    "undarkened_tex".to_string(),
                ],
                ..Default::default()
            },
        )
        .unwrap();

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

        let scene_comp_mat = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: SCENE_FRAGMENT_SHADER },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
                    )),
                    ..Default::default()
                },
                textures: vec![
                    "amb_tex".to_string(),
                    "glow_tex".to_string(),
                    "scene_comp_tex".to_string(),
                ],
                ..Default::default()
            },
        ).unwrap();

        let final_comp_mat = load_material(
            ShaderSource::Glsl { vertex: VERTEX_SHADER, fragment: COMPOSITE_FRAGMENT_SHADER },
            MaterialParams {
                textures: vec![
                    "scene_comp_tex".to_string(),
                    "spot_tex".to_string(),
                    "final_comp_tex".to_string(),
                ],
                ..Default::default()
            },
        ).unwrap();

        Self {
            scene_rt: make_render_target(),
            ambient_rt: make_render_target(),
            glow_rt: make_render_target(),
            undarkened_rt: make_render_target(),
            spot_rt: make_render_target(),
            mask_rt: make_render_target(),
            scene_comp_rt: make_render_target(),
            final_comp_rt: make_render_target(),
            ambient_mat,
            glow_mat,
            undarkened_mat,
            spot_mat,
            scene_comp_mat,
            final_comp_mat,
            light_bufffers: [LightBuffer::default(); MAX_LIGHTS],
            glow_bufffers: [GlowBuffer::default(); MAX_LIGHTS],
        }
    }

    /// Applies darkness to the scene.
    pub fn run_ambient_pass(
        &mut self,
        darkness: f32,
    ) {
        LightSystem::clear_cam(&self.ambient_rt);

        self.ambient_mat.set_texture("tex", self.scene_rt.texture.clone());
        self.ambient_mat.set_uniform("Darkness", darkness);

        gl_use_material(&self.ambient_mat);
        draw_texture_ex(
            &self.scene_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams::default(),
        );
        gl_use_default_material();
    }

    /// Renders glow textures per-layer in the room.
    pub fn run_glow_pass(
        &mut self,
        render_cam: &Camera2D,
        glows: Vec<(&Glow, Vec2)>,
        asset_manager: &mut AssetManager,
    ) {
        LightSystem::clear_cam(&self.glow_rt);
        self.clear_glow_buffers();
        if glows.is_empty() {
            return;
        }

        self.glow_mat.set_texture("scene_tex", self.scene_rt.texture.clone());

        for (i, (glow, world_pos)) in glows.iter().take(MAX_LIGHTS).enumerate() {
            if let Some(id) = asset_manager.get_or_load(&glow.sprite) {
                let tex = asset_manager.get_texture_from_id(id).clone();
                self.glow_mat.set_texture(&format!("tex_mask{}", i), tex);

                let screen_pos = render_cam.world_to_screen(*world_pos);
                let buffer = &mut self.glow_bufffers[i];
                buffer.pos = screen_pos;
                buffer.pos = screen_pos;
                buffer.color = glow.color;
                buffer.intensity = glow.intensity;
                buffer.brightness = glow.brightness;
                buffer.emission = glow.emission;

                // Texture dimensions
                if let Some((w, h)) = asset_manager.texture_size(id) {
                    buffer.mask_size = vec2(
                        world_distance_to_screen(render_cam, w),
                        world_distance_to_screen(render_cam, h),
                    );
                }
            }
        }
        
        self.glow_mat.set_uniform("GlowCount", glows.len().min(MAX_LIGHTS) as i32);
        self.glow_mat.set_uniform_array("Brightness", &self.glow_bufffers.map(|g| g.brightness));
        self.glow_mat.set_uniform_array("Intensity", &self.glow_bufffers.map(|g| g.intensity));
        self.glow_mat.set_uniform_array("Color", &self.glow_bufffers.map(|g| g.color));
        self.glow_mat.set_uniform_array("Emission", &self.glow_bufffers.map(|g| g.emission));
        self.glow_mat.set_uniform_array("maskPos", &self.glow_bufffers.map(|g| g.pos));
        self.glow_mat.set_uniform_array("maskSize", &self.glow_bufffers.map(|g| g.mask_size));
        self.glow_mat.set_uniform("screenWidth", screen_width());
        self.glow_mat.set_uniform("screenHeight", screen_height());
        
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

    /// Renders the scene without applying darkness so the lighting pass can operate
    /// on an undimmed texture.
    pub fn run_undarkened_pass(&mut self) {
        let cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom:   vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(self.undarkened_rt.clone()),
            ..Default::default()
        };
        set_camera(&cam);

        self.undarkened_mat.set_texture("scene_tex", self.scene_rt.texture.clone());
        self.undarkened_mat.set_texture("glow_tex", self.glow_rt.texture.clone());
        self.undarkened_mat.set_texture("undarkened_tex", self.undarkened_rt.texture.clone());

        gl_use_material(&self.undarkened_mat);
        draw_texture_ex(
            &self.undarkened_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams::default(),
        );
        gl_use_default_material();
    }

    /// Renders spotlights using the undarkened scene texture.
    pub fn run_spotlight_pass(
        &mut self,
        render_cam: &Camera2D, 
        lights: Vec<(Vec2, Light)>,
        darkness: f32,
    ) {
        LightSystem::clear_cam(&self.spot_rt);
        self.clear_light_buffers();

        if !lights.is_empty() {
            let light_count = lights.len(); 

            for i in 0..light_count {
                let (pos, l) = &lights[i];
                let world_pos = *pos + l.pos;
                let mut buffer = self.light_bufffers[i];

                buffer.pos = render_cam.world_to_screen(world_pos);
                buffer.radius = world_distance_to_screen(render_cam, l.radius);
                buffer.spread = world_distance_to_screen(render_cam, l.spread);
                buffer.color = l.color;
                buffer.intensity = l.intensity;
                buffer.alpha = l.alpha;
                buffer.brightness = l.brightness;
            }

            self.spot_mat.set_texture("tex", self.undarkened_rt.texture.clone());
            self.spot_mat.set_texture("light_mask", self.mask_rt.texture.clone());

            self.spot_mat.set_uniform("LightCount", light_count as i32);
            self.spot_mat.set_uniform_array("LightPos", &self.light_bufffers.map(|g| g.pos));
            self.spot_mat.set_uniform_array("LightColor", &self.light_bufffers.map(|g| g.color));
            self.spot_mat.set_uniform_array("LightIntensity", &self.light_bufffers.map(|g| g.intensity));
            self.spot_mat.set_uniform_array("LightRadius", &self.light_bufffers.map(|g| g.radius));
            self.spot_mat.set_uniform_array("LightSpread", &self.light_bufffers.map(|g| g.spread));
            self.spot_mat.set_uniform_array("LightAlpha", &self.light_bufffers.map(|g| g.alpha));
            self.spot_mat.set_uniform_array("LightBrightness", &self.light_bufffers.map(|g| g.brightness));
            self.spot_mat.set_uniform("ScreenWidth", screen_width());
            self.spot_mat.set_uniform("ScreenHeight", screen_height());
            self.spot_mat.set_uniform("Darkness", darkness);

            gl_use_material(&self.spot_mat);
            draw_texture_ex(
                &self.spot_rt.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams::default(),
            );
            gl_use_default_material();
        }
    }

    /// Composites the per-layer room textures.
    pub fn run_scene_pass(&mut self) {
        let scene_comp_cam = Camera2D {
            target: vec2(screen_width() * 0.5, screen_height() * 0.5),
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(self.scene_comp_rt.clone()),
            ..Default::default()
        };

        set_camera(&scene_comp_cam);

        self.scene_comp_mat.set_texture("amb_tex", self.ambient_rt.texture.clone());
        self.scene_comp_mat.set_texture("glow_tex", self.glow_rt.texture.clone());
        self.scene_comp_mat.set_texture("scene_comp_tex", self.scene_comp_rt.texture.clone());

        gl_use_material(&self.scene_comp_mat);
        draw_texture_ex(
            &self.scene_comp_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams::default(),
        );
        gl_use_default_material();
    }

    /// The last composite stage for rendering a room before post-processing.
    pub fn run_final_pass(&mut self) {
        set_default_camera();

        self.final_comp_mat.set_texture("scene_comp_tex", self.scene_comp_rt.texture.clone());
        self.final_comp_mat.set_texture("spot_tex", self.spot_rt.texture.clone());
        self.final_comp_mat.set_texture("final_comp_tex", self.final_comp_rt.texture.clone());

        gl_use_material(&self.final_comp_mat);
        draw_texture_ex(
            &self.final_comp_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams::default(),
        );
        gl_use_default_material();
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
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            render_target: Some(rt.clone()),
            ..Default::default()
        };
        set_camera(&cam);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        cam
    }

    /// Reset the per‑frame light buffers.
    pub fn clear_light_buffers(&mut self) {
        self.light_bufffers
            .iter_mut()
            .for_each(|slot| *slot = LightBuffer::default());
    }

    /// Reset the per‑frame glow buffers.
    pub fn clear_glow_buffers(&mut self) {
        self.glow_bufffers
            .iter_mut()
            .for_each(|slot| *slot = GlowBuffer::default());
    }
}

/// Distance conversion for shader uniforms.
pub fn world_distance_to_screen(cam: &Camera2D, distance: f32) -> f32 {
    let scale = cam.zoom.x * screen_width() * 0.5; 
    (distance * scale).abs()
}