// engine_core/src/rendering/render_system_wgpu.rs
// Wgpu-specific implementation of RenderSystem.
// NOTE: Multi-pass rendering temporarily disabled while rewiring codebase.

use crate::prelude::*;
use bishop::prelude::*;

// TODO: Re-enable multi-pass rendering imports
// use bishop::prelude::*;
// use bishop::wgpu::{
//     AmbientMaterial, BishopRenderTarget, FinalCompositeMaterial,
//     FullscreenQuadRenderer, GlowData, GlowMaterial, GlowUniforms, SceneCompositeMaterial,
//     SpotLightData, SpotMaterial, SpotUniforms, UndarkenedMaterial, WgpuContext, Texture
// };

/// Max lights per layer.
pub const MAX_LIGHTS: usize = 10;

/// Simplified render system for direct drawing.
/// Multi-pass rendering is temporarily disabled.
pub struct RenderSystem {
    /// Current render target width
    pub rt_width: f32,
    /// Current render target height
    pub rt_height: f32,
    /// Time spent rendering last frame (ms)
    pub render_time_ms: f32,

    // TODO: Re-enable multi-pass rendering fields
    // pub scene_rt: BishopRenderTarget,
    // pub ambient_rt: BishopRenderTarget,
    // pub glow_rt: BishopRenderTarget,
    // pub undarkened_rt: BishopRenderTarget,
    // pub spot_rt: BishopRenderTarget,
    // pub mask_rt: BishopRenderTarget,
    // pub scene_comp_rt: BishopRenderTarget,
    // pub final_comp_rt: BishopRenderTarget,
    // ambient_mat: AmbientMaterial,
    // glow_mat: GlowMaterial,
    // undarkened_mat: UndarkenedMaterial,
    // spot_mat: SpotMaterial,
    // scene_comp_mat: SceneCompositeMaterial,
    // final_comp_mat: FinalCompositeMaterial,
    // fullscreen_quad: FullscreenQuadRenderer,
    // placeholder_texture: Texture,
    // placeholder_view: TextureView,
    // placeholder_sampler: Sampler,
}

impl RenderSystem {
    /// Create a new simplified render system with the given dimensions.
    fn new(width: f32, height: f32) -> Self {
        Self {
            rt_width: width,
            rt_height: height,
            render_time_ms: 0.0,
        }
    }

    /// Create a new render system with render targets sized for the given grid size.
    pub fn with_grid_size(grid_size: f32) -> Self {
        let width = world_virtual_width(grid_size);
        let height = world_virtual_height(grid_size);
        Self::new(width, height)
    }

    /// Create a new render system with default grid size (16.0).
    pub fn with_default_grid_size() -> Self {
        Self::with_grid_size(16.0)
    }

    // TODO: Re-enable multi-pass rendering constructor
    // pub fn with_grid_size_wgpu(ctx: &WgpuContext, grid_size: f32) -> Self { ... }

    // TODO: Re-enable multi-pass rendering passes
    // pub fn run_ambient_pass<C: BishopContext>(&mut self, ctx: &mut C, darkness: f32) { ... }
    // pub fn run_glow_pass<C: BishopContext>(...) { ... }
    // pub fn run_undarkened_pass<C: BishopContext>(&mut self, ctx: &mut C) { ... }
    // pub fn run_spotlight_pass<C: BishopContext>(...) { ... }
    // pub fn run_scene_pass<C: BishopContext>(&mut self, ctx: &mut C) { ... }
    // pub fn run_final_pass<C: BishopContext>(&mut self, ctx: &mut C) { ... }

    /// Presents the final visual of the game with hybrid scaling.
    /// Uses fractional scale to fill screen while maintaining aspect ratio, minimizing letterboxing.
    pub fn present_game<C: BishopContext>(
        &self,
        ctx: &mut C, 
    ) {
        // TODO Re-enable
        ctx.set_default_camera();
        // let tex = &self.final_comp_rt.texture;

        // let virt_w = self.rt_width;
        // let virt_h = self.rt_height;
        // let win_w = ctx.screen_width();
        // let win_h = ctx.screen_height();

        // // Hybrid scaling: use fractional scale to fill screen while maintaining aspect ratio
        // let scale = (win_w / virt_w).min(win_h / virt_h);
        // let scaled_w = virt_w * scale;
        // let scaled_h = virt_h * scale;

        // let offset_x = ((win_w - scaled_w) / 2.0).floor();
        // let offset_y = ((win_h - scaled_h) / 2.0).floor();

        // ctx.draw_texture_ex(
        //     tex,
        //     offset_x,
        //     offset_y,
        //     Color::WHITE,
        //     DrawTextureParams {
        //         dest_size: Some(Vec2::new(scaled_w, scaled_h)),
        //         ..Default::default()
        //     },
        // );
    }

    /// Re-creates every render target with the supplied size.
    pub fn resize(&mut self, width: u32, height: u32) {
        // TODO: Re-implement
        // self.rt_width = width as f32;
        // self.rt_height = height as f32;

        // let make = || {
        //     let rt = render_target(width, height);
        //     rt.texture.set_filter(FilterMode::Nearest);
        //     rt
        // };

        // self.scene_rt = make();
        // self.ambient_rt = make();
        // self.glow_rt = make();
        // self.undarkened_rt = make();
        // self.spot_rt = make();
        // self.mask_rt = make();
        // self.scene_comp_rt = make();
        // self.final_comp_rt = make();

        // // Reset the mask cam
        // self.init_mask_cam();
    }

    /// Resizes render targets to match the camera zoom.
    pub fn resize_for_camera(&mut self, zoom: Vec2) {
        let required_width = (2.0 / zoom.x).round();
        let required_height = (2.0 / zoom.y).round();

        if required_width != self.rt_width || required_height != self.rt_height {
            self.rt_width = required_width;
            self.rt_height = required_height;
        }
    }

    /// Resizes render targets to match window size if they don't already.
    pub fn resize_to_window<C: BishopContext>(&mut self, ctx: &mut C) {
        let win_w = ctx.screen_width();
        let win_h = ctx.screen_height();

        if win_w != self.rt_width || win_h != self.rt_height {
            self.rt_width = win_w;
            self.rt_height = win_h;
        }
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::with_default_grid_size()
    }
}
