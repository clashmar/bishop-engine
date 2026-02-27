// engine_core/src/rendering/render_system_wgpu.rs
// Wgpu-specific implementation of RenderSystem.

use crate::prelude::*;
use bishop::prelude::*;
use bishop::wgpu::{
    wgpu_crate, AmbientMaterial, BishopRenderTarget, FinalCompositeMaterial,
    FullscreenQuadRenderer, GlowData, GlowMaterial, GlowUniforms, SceneCompositeMaterial,
    SpotLightData, SpotMaterial, SpotUniforms, UndarkenedMaterial, WgpuContext,
};

/// Max lights per layer.
pub const MAX_LIGHTS: usize = 10;

pub struct RenderSystem {
    // Render targets
    pub scene_rt: BishopRenderTarget,
    pub ambient_rt: BishopRenderTarget,
    pub glow_rt: BishopRenderTarget,
    pub undarkened_rt: BishopRenderTarget,
    pub spot_rt: BishopRenderTarget,
    pub mask_rt: BishopRenderTarget,
    pub scene_comp_rt: BishopRenderTarget,
    pub final_comp_rt: BishopRenderTarget,
    /// Materials
    ambient_mat: AmbientMaterial,
    glow_mat: GlowMaterial,
    undarkened_mat: UndarkenedMaterial,
    spot_mat: SpotMaterial,
    scene_comp_mat: SceneCompositeMaterial,
    final_comp_mat: FinalCompositeMaterial,
    /// Fullscreen quad renderer for post-processing
    fullscreen_quad: FullscreenQuadRenderer,
    /// Time spent rendering last frame (ms)
    pub render_time_ms: f32,
    /// Current render target dimensions
    rt_width: f32,
    rt_height: f32,
    /// Placeholder texture for unused glow masks
    placeholder_texture: wgpu_crate::Texture,
    placeholder_view: wgpu_crate::TextureView,
    placeholder_sampler: wgpu_crate::Sampler,
}

impl RenderSystem {
    /// Create a new render system with render targets sized for the given grid size.
    /// Must be called after wgpu context is initialized.
    pub fn with_grid_size_wgpu(ctx: &WgpuContext, grid_size: f32) -> Self {
        let width = world_virtual_width(grid_size) as u32;
        let height = world_virtual_height(grid_size) as u32;

        let device = ctx.device();
        let format = ctx.surface_format();
        let layout_arc = ctx.render_target_bind_group_layout_arc();

        let make_render_target = |w: u32, h: u32| {
            BishopRenderTarget::new(device, layout_arc.clone(), w, h, format, FilterMode::Nearest)
        };

        let camera_bind_group_layout = ctx.fullscreen_quad_renderer().camera_bind_group_layout();

        let ambient_mat = AmbientMaterial::new(device, format, camera_bind_group_layout);
        let glow_mat = GlowMaterial::new(device, format, camera_bind_group_layout);
        let undarkened_mat = UndarkenedMaterial::new(device, format, camera_bind_group_layout);
        let spot_mat = SpotMaterial::new(device, format, camera_bind_group_layout);
        let scene_comp_mat = SceneCompositeMaterial::new(device, format, camera_bind_group_layout);
        let final_comp_mat = FinalCompositeMaterial::new(device, format, camera_bind_group_layout);

        let fullscreen_quad = FullscreenQuadRenderer::new(device);

        // Create a 1x1 transparent placeholder texture for unused glow masks
        let (placeholder_texture, placeholder_view, placeholder_sampler) =
            Self::create_placeholder_texture(device);

        Self {
            scene_rt: make_render_target(width, height),
            ambient_rt: make_render_target(width, height),
            glow_rt: make_render_target(width, height),
            undarkened_rt: make_render_target(width, height),
            spot_rt: make_render_target(width, height),
            mask_rt: make_render_target(width, height),
            scene_comp_rt: make_render_target(width, height),
            final_comp_rt: make_render_target(width, height),
            ambient_mat,
            glow_mat,
            undarkened_mat,
            spot_mat,
            scene_comp_mat,
            final_comp_mat,
            fullscreen_quad,
            render_time_ms: 0.0,
            rt_width: width as f32,
            rt_height: height as f32,
            placeholder_texture,
            placeholder_view,
            placeholder_sampler,
        }
    }

    /// Create a new render system with default grid size.
    pub fn with_grid_size(_grid_size: f32) -> Self {
        panic!("RenderSystem::with_grid_size requires WgpuContext. Use with_grid_size_wgpu instead.");
    }

    /// Create a new render system with default grid size (16.0).
    pub fn new() -> Self {
        panic!("RenderSystem::new requires WgpuContext. Use with_grid_size_wgpu instead.");
    }

    fn create_placeholder_texture(
        device: &wgpu_crate::Device,
    ) -> (wgpu_crate::Texture, wgpu_crate::TextureView, wgpu_crate::Sampler) {
        let texture = device.create_texture(&wgpu_crate::TextureDescriptor {
            label: Some("placeholder_texture"),
            size: wgpu_crate::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu_crate::TextureDimension::D2,
            format: wgpu_crate::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu_crate::TextureUsages::TEXTURE_BINDING | wgpu_crate::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu_crate::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu_crate::SamplerDescriptor {
            label: Some("placeholder_sampler"),
            address_mode_u: wgpu_crate::AddressMode::ClampToEdge,
            address_mode_v: wgpu_crate::AddressMode::ClampToEdge,
            address_mode_w: wgpu_crate::AddressMode::ClampToEdge,
            mag_filter: wgpu_crate::FilterMode::Nearest,
            min_filter: wgpu_crate::FilterMode::Nearest,
            ..Default::default()
        });

        (texture, view, sampler)
    }

    /// Applies darkness to the scene.
    pub fn run_ambient_pass<C: BishopContext>(&mut self, _ctx: &mut C, _darkness: f32) {
        // TODO: Implement wgpu ambient pass
        // For now, this is a no-op
    }

    /// Renders glow textures per-layer in the room.
    pub fn run_glow_pass<C: BishopContext>(
        &mut self,
        _ctx: &mut C,
        _render_cam: &Camera2D,
        _glows: Vec<(&Glow, Vec2)>,
        _asset_manager: &mut AssetManager,
    ) {
        // TODO: Implement wgpu glow pass
        // For now, this is a no-op
    }

    /// Renders the scene without applying darkness so the lighting pass can operate
    /// on an undimmed texture.
    pub fn run_undarkened_pass<C: BishopContext>(&mut self, _ctx: &mut C) {
        // TODO: Implement wgpu undarkened pass
        // For now, this is a no-op
    }

    /// Renders spotlights using the undarkened scene texture.
    pub fn run_spotlight_pass<C: BishopContext>(
        &mut self,
        _ctx: &mut C,
        _render_cam: &Camera2D,
        _lights: Vec<(Vec2, Light)>,
        _darkness: f32,
    ) {
        // TODO: Implement wgpu spotlight pass
        // For now, this is a no-op
    }

    /// Composites the per-layer room textures.
    pub fn run_scene_pass<C: BishopContext>(&mut self, _ctx: &mut C) {
        // TODO: Implement wgpu scene pass
        // For now, this is a no-op
    }

    /// The last composite stage for rendering a room before post-processing.
    pub fn run_final_pass<C: BishopContext>(&mut self, _ctx: &mut C) {
        // TODO: Implement wgpu final pass
        // For now, this is a no-op
    }

    /// Presents the final visual of the game with hybrid scaling.
    pub fn present_game<C: BishopContext>(&self, ctx: &mut C) {
        ctx.set_default_camera();

        let virt_w = self.rt_width;
        let virt_h = self.rt_height;
        let win_w = ctx.screen_width();
        let win_h = ctx.screen_height();

        // Hybrid scaling: use fractional scale to fill screen while maintaining aspect ratio
        let scale = (win_w / virt_w).min(win_h / virt_h);
        let scaled_w = virt_w * scale;
        let scaled_h = virt_h * scale;

        let offset_x = ((win_w - scaled_w) / 2.0).floor();
        let offset_y = ((win_h - scaled_h) / 2.0).floor();

        // TODO: Draw the final composite render target
        // For now, draw a placeholder rectangle
        ctx.draw_rectangle(offset_x, offset_y, scaled_w, scaled_h, Color::DARKGRAY);
    }

    /// Presents the render target directly at 1:1 for window-sized targets.
    pub fn present<C: BishopContext>(&self, ctx: &mut C) {
        ctx.set_default_camera();
        // TODO: Draw the final composite render target
        // For now, draw a placeholder rectangle
        ctx.draw_rectangle(0.0, 0.0, self.rt_width, self.rt_height, Color::DARKGRAY);
    }

    /// Sets the mask render target background to white.
    pub fn init_mask_cam<C: BishopContext>(&self, _ctx: &mut C) {
        // TODO: Implement wgpu mask cam initialization
        // For now, this is a no-op
    }

    /// Sets, clears and returns the scene camera.
    pub fn clear_scene_cam<C: BishopContext>(
        &self,
        ctx: &mut C,
        render_cam: &Camera2D,
    ) -> Camera2D {
        let scene_cam = Camera2D {
            target: render_cam.target,
            zoom: render_cam.zoom,
            ..Default::default()
        };

        ctx.set_camera(&scene_cam);
        ctx.clear_background(Color::TRANSPARENT);
        scene_cam
    }

    /// Sets, clears the given render target and returns the camera for it.
    pub fn clear_cam<C: BishopContext>(
        &self,
        ctx: &mut C,
        _rt: &BishopRenderTarget,
    ) -> Camera2D {
        let cam = Camera2D {
            target: vec2(self.rt_width * 0.5, self.rt_height * 0.5),
            zoom: vec2(2.0 / self.rt_width, 2.0 / self.rt_height),
            ..Default::default()
        };
        ctx.set_camera(&cam);
        ctx.clear_background(Color::TRANSPARENT);
        cam
    }

    /// Clears both composite render targets before a new frame.
    pub fn clear_composite_cams<C: BishopContext>(&self, ctx: &mut C) {
        // TODO: Implement proper clearing of composite render targets
        let cam = Camera2D {
            target: vec2(self.rt_width * 0.5, self.rt_height * 0.5),
            zoom: vec2(2.0 / self.rt_width, 2.0 / self.rt_height),
            ..Default::default()
        };
        ctx.set_camera(&cam);
        ctx.clear_background(Color::TRANSPARENT);
    }

    /// Resizes render targets to match the camera zoom.
    pub fn resize_for_camera(&mut self, zoom: Vec2) {
        let required_width = (2.0 / zoom.x).round() as u32;
        let required_height = (2.0 / zoom.y).round() as u32;

        if required_width != self.rt_width as u32 || required_height != self.rt_height as u32 {
            // TODO: Implement wgpu resize
            self.rt_width = required_width as f32;
            self.rt_height = required_height as f32;
        }
    }

    /// Resizes render targets to match window size if they don't already.
    pub fn resize_to_window<C: BishopContext>(&mut self, ctx: &mut C) {
        let win_w = ctx.screen_width() as u32;
        let win_h = ctx.screen_height() as u32;

        if win_w != self.rt_width as u32 || win_h != self.rt_height as u32 {
            // TODO: Implement wgpu resize
            self.rt_width = win_w as f32;
            self.rt_height = win_h as f32;
        }
    }
}
