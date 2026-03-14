// game/src/diagnostics/overlay.rs
//! In-game diagnostics overlay toggled with F3/F4.

use crate::game_instance::GameInstance;
use engine_core::prelude::*;
use bishop::prelude::*;

/// Detail level for the diagnostics overlay.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum OverlayDetailLevel {
    /// Overlay is hidden.
    #[default]
    Off,
    /// Show basic metrics (FPS only).
    Basic,
    /// Show detailed metrics.
    Detailed,
}

impl OverlayDetailLevel {
    /// Cycle to the next detail level.
    pub fn cycle(self) -> Self {
        match self {
            OverlayDetailLevel::Off => OverlayDetailLevel::Basic,
            OverlayDetailLevel::Basic => OverlayDetailLevel::Detailed,
            OverlayDetailLevel::Detailed => OverlayDetailLevel::Off,
        }
    }
}

/// Runtime diagnostics overlay for the game.
pub struct DiagnosticsOverlay {
    /// Current detail level.
    pub detail_level: OverlayDetailLevel,
    /// Metrics collector.
    collector: DiagnosticsCollector,
    /// Cached metrics for display.
    cached_fps: f32,
    cached_frame_time: f32,
    cached_render_time: f32,
    cached_entity_count: usize,
    cached_texture_count: usize,
    cached_script_instances: usize,
    cached_listener_count: usize,
    cached_script_id_count: usize,
    cached_sprite_id_count: usize,
}

impl Default for DiagnosticsOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsOverlay {
    pub fn new() -> Self {
        Self {
            detail_level: OverlayDetailLevel::Off,
            collector: DiagnosticsCollector::new(),
            cached_fps: 0.0,
            cached_frame_time: 0.0,
            cached_render_time: 0.0,
            cached_entity_count: 0,
            cached_texture_count: 0,
            cached_script_instances: 0,
            cached_listener_count: 0,
            cached_script_id_count: 0,
            cached_sprite_id_count: 0,
        }
    }

    /// Toggle the overlay on/off.
    pub fn toggle(&mut self) {
        self.detail_level = if self.detail_level == OverlayDetailLevel::Off {
            OverlayDetailLevel::Basic
        } else {
            OverlayDetailLevel::Off
        };
    }

    /// Cycle through detail levels.
    pub fn cycle_detail(&mut self) {
        self.detail_level = self.detail_level.cycle();
    }

    /// Update frame timing metrics.
    pub fn update(&mut self, dt: f32) {
        self.collector.record_frame(dt);
        self.cached_fps = self.collector.frame_metrics.fps;
        self.cached_frame_time = self.collector.frame_metrics.avg_frame_time_ms;
    }

    /// Pulls current metrics from the game instance and render system.
    pub fn update_from_game(&mut self, game_instance: &GameInstance, render_time_ms: f32) {
        let game = &game_instance.game;
        self.cached_entity_count = game.ecs.get_store::<Transform>().data.len();
        self.cached_texture_count = game.asset_manager.texture_count();
        self.cached_script_instances = game.script_manager.instances.len();
        self.cached_listener_count = game.script_manager.event_bus.listener_count();
        self.cached_script_id_count = game.script_manager.script_id_to_path.len();
        self.cached_sprite_id_count = game.asset_manager.sprite_id_to_path.len();
        self.cached_render_time = render_time_ms;
    }

    /// Handle input for toggling the overlay.
    pub fn handle_input(
        &mut self, 
        ctx: &mut impl BishopContext,
    ) {
        if ctx.is_key_pressed(KeyCode::F3) {
            self.toggle();
        }
        if ctx.is_key_pressed(KeyCode::F4) {
            self.cycle_detail();
        }
    }

    /// Draw the overlay.
    pub fn draw<C: BishopContext>(
        &self,
        ctx: &mut C,
    ) {
        if self.detail_level == OverlayDetailLevel::Off {
            return;
        }

        const PADDING: f32 = 10.0;
        const LINE_HEIGHT: f32 = 18.0;
        const FONT_SIZE: f32 = 14.0;
        const BG_ALPHA: f32 = 0.7;

        let mut lines: Vec<String> = Vec::new();

        // FPS line
        let fps_str = format!("FPS: {:.1}", self.cached_fps);
        lines.push(fps_str);

        if self.detail_level == OverlayDetailLevel::Detailed {
            lines.push(format!("Frame: {:.2} ms", self.cached_frame_time));
            lines.push(format!("Render: {:.2} ms", self.cached_render_time));
            lines.push(format!("Entities: {}", self.cached_entity_count));
            lines.push(format!("Textures: {}", self.cached_texture_count));
            lines.push(format!("Sprite IDs: {}", self.cached_sprite_id_count));
            lines.push(format!("Script IDs: {}", self.cached_script_id_count));
            lines.push(format!("Script Instances: {}", self.cached_script_instances));
            lines.push(format!("Listeners: {}", self.cached_listener_count));
        }

        // Calculate background size
        let max_width = lines
            .iter()
            .map(|s| ctx.measure_text(s, FONT_SIZE).width)
            .fold(0.0_f32, f32::max);

        let bg_width = max_width + PADDING * 2.0;
        let bg_height = lines.len() as f32 * LINE_HEIGHT + PADDING * 2.0;

        // Draw background
        ctx.draw_rectangle(
            PADDING,
            PADDING,
            bg_width,
            bg_height,
            Color::new(0.0, 0.0, 0.0, BG_ALPHA),
        );

        // Draw text
        let fps_color = Self::fps_color(self.cached_fps);

        for (i, line) in lines.iter().enumerate() {
            let color = if i == 0 { fps_color } else { Color::WHITE };
            let y = PADDING * 2.0 + LINE_HEIGHT * i as f32;
            ctx.draw_text(line, PADDING * 2.0, y + FONT_SIZE, FONT_SIZE, color);
        }
    }

    fn fps_color(fps: f32) -> Color {
        if fps >= 55.0 {
            Color::GREEN
        } else if fps >= 30.0 {
            Color::YELLOW
        } else {
            Color::RED
        }
    }
}
