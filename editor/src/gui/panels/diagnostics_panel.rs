// editor/src/gui/panels/diagnostics_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::editor_global::with_command_manager;
use crate::Editor;
use engine_core::prelude::*;
use bishop::prelude::*;

const ROW_HEIGHT: f32 = 16.0;
const SECTION_SPACING: f32 = 8.0;
const SCROLL_SPEED: f32 = 24.0;
const SCROLLBAR_W: f32 = 6.0;
const TOP_PADDING: f32 = 8.0;
const LEFT_PADDING: f32 = 8.0;
const FONT_SIZE: f32 = 13.0;
const HEADER_FONT_SIZE: f32 = 14.0;
const UPDATE_INTERVAL: f32 = 0.5; // Update every 500ms

pub struct DiagnosticsPanel {
    scroll_y: f32,
    collector: DiagnosticsCollector,
    last_snapshot: Option<DiagnosticsSnapshot>,
    time_since_update: f32,
}

impl DiagnosticsPanel {
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            collector: DiagnosticsCollector::new(),
            last_snapshot: None,
            time_since_update: UPDATE_INTERVAL, // Force immediate update
        }
    }

    fn collect_metrics(&self, editor: &Editor) -> DiagnosticsSnapshot {
        let game = &editor.game;

        // Asset metrics
        let asset_metrics = AssetMetrics {
            texture_count: game.asset_manager.texture_count(),
            tile_def_count: game.asset_manager.tile_def_count(),
            sprite_id_count: game.asset_manager.sprite_id_to_path.len(),
            script_id_count: game.script_manager.script_id_to_path.len(),
        };

        // Script metrics
        let script_metrics = ScriptMetrics {
            loaded_count: game.script_manager.table_defs.len(),
            instance_count: game.script_manager.instances.len(),
            event_listener_count: game.script_manager.event_bus.listener_count(),
            ref_counts: game
                .script_manager
                .script_id_to_path
                .keys()
                .map(|id| (id.0, game.script_manager.get_ref_count(*id)))
                .collect(),
        };

        // ECS metrics
        let ecs = &game.ecs;
        let entity_count = ecs.get_store::<Transform>().data.len();
        let ecs_metrics = EcsMetrics {
            entity_count,
            component_store_count: ecs.stores.len(),
            components_by_type: std::collections::HashMap::new(),
        };

        // Command metrics
        let command_metrics = with_command_manager(|cm| CommandMetrics {
            undo_stack_size: cm.undo_stack_len(),
            redo_stack_size: cm.redo_stack_len(),
            pending_size: cm.pending_len(),
        });

        DiagnosticsSnapshot {
            frame: self.collector.frame_metrics.clone(),
            assets: asset_metrics,
            scripts: script_metrics,
            ecs: ecs_metrics,
            commands: command_metrics,
        }
    }

    /// Returns true if the row is fully visible within the clip rect.
    fn is_visible(&self, y: f32, clip: &Rect) -> bool {
        let top = clip.y + 2.0;
        let bottom = clip.y + clip.h - ROW_HEIGHT;
        y >= top && y <= bottom
    }

    fn draw_section_header(
        &self, 
        ctx: &mut WgpuContext,
        label: &str, 
        y: f32, 
        rect: &Rect
    ) -> f32 {
        if self.is_visible(y, rect) {
            ctx.draw_text(label, rect.x + LEFT_PADDING, y + HEADER_FONT_SIZE, HEADER_FONT_SIZE, Color::YELLOW);
        }
        y + ROW_HEIGHT + 4.0
    }

    fn draw_row(
        &self, 
        ctx: &mut WgpuContext,
        label: &str, 
        value: &str, 
        y: f32, 
        rect: &Rect, 
        color: Color
    ) -> f32 {
        if self.is_visible(y, rect) {
            ctx.draw_text(label, rect.x + LEFT_PADDING + 8.0, y + FONT_SIZE, FONT_SIZE, Color::GREY);
            let value_x = rect.x + rect.w * 0.55;
            ctx.draw_text(value, value_x, y + FONT_SIZE, FONT_SIZE, color);
        }
        y + ROW_HEIGHT
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

pub const DIAGNOSTICS_PANEL: &str = "Diagnostics";

impl PanelDefinition for DiagnosticsPanel {
    fn title(&self) -> &'static str {
        DIAGNOSTICS_PANEL
    }

    fn default_rect(&self, ctx: &WgpuContext,) -> Rect {
        Rect::new(ctx.screen_width() - 280., 60., 260., 360.)
    }

    fn draw(
        &mut self, 
        ctx: &mut WgpuContext,
        rect: Rect, 
        editor: &mut Editor, 
        blocked: bool
    ) {
        let mouse: Vec2 = ctx.mouse_position().into();
        let dt = ctx.get_frame_time();

        // Update frame timing continuously
        self.collector.record_frame(dt);

        // Update full snapshot at intervals to reduce overhead
        self.time_since_update += dt;
        if self.time_since_update >= UPDATE_INTERVAL {
            self.time_since_update = 0.0;
            self.last_snapshot = Some(self.collect_metrics(editor));
        }

        // Scroll input
        if !blocked && rect.contains(mouse) {
            let (_, wheel_y) = ctx.mouse_wheel();
            self.scroll_y += wheel_y * SCROLL_SPEED;
        }

        // Draw content
        let snapshot = match &self.last_snapshot {
            Some(s) => s.clone(),
            None => return,
        };

        // Calculate content height
        let mut content_height = TOP_PADDING;
        content_height += ROW_HEIGHT + SECTION_SPACING; // Warnings header
        let warnings = self.collector.generate_warnings(&snapshot);
        content_height += warnings.len().max(1) as f32 * ROW_HEIGHT;
        content_height += SECTION_SPACING;

        content_height += ROW_HEIGHT + SECTION_SPACING; // Performance
        content_height += 5.0 * ROW_HEIGHT; // FPS, avg, min, max, render
        content_height += SECTION_SPACING;

        content_height += ROW_HEIGHT + SECTION_SPACING; // Assets
        content_height += 3.0 * ROW_HEIGHT; // Textures, Tile Defs, Sprite IDs
        content_height += SECTION_SPACING;

        content_height += ROW_HEIGHT + SECTION_SPACING; // Scripts
        content_height += 4.0 * ROW_HEIGHT; // Script IDs, Loaded, Instances, Listeners
        content_height += SECTION_SPACING;

        content_height += ROW_HEIGHT + SECTION_SPACING; // ECS
        content_height += 2.0 * ROW_HEIGHT;
        content_height += SECTION_SPACING;

        content_height += ROW_HEIGHT + SECTION_SPACING; // Undo/Redo
        content_height += 3.0 * ROW_HEIGHT;
        content_height += TOP_PADDING;

        let scroll_range = (content_height - rect.h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        let mut y = rect.y + self.scroll_y + TOP_PADDING;

        // Warnings section
        y = self.draw_section_header(ctx, "Warnings", y, &rect);
        if warnings.is_empty() {
            y = self.draw_row(ctx, "Status", "OK", y, &rect, Color::GREEN);
        } else {
            for warning in &warnings {
                y = self.draw_row(ctx, "!", &warning.description(), y, &rect, Color::RED);
            }
        }
        y += SECTION_SPACING;

        // Performance section
        y = self.draw_section_header(ctx, "Performance", y, &rect);
        let fps = snapshot.frame.fps;
        y = self.draw_row(ctx, "FPS", &format!("{:.1}", fps), y, &rect, Self::fps_color(fps));
        y = self.draw_row(ctx, "Avg", &format!("{:.2} ms", snapshot.frame.avg_frame_time_ms), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Min", &format!("{:.2} ms", snapshot.frame.min_frame_time_ms), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Max", &format!("{:.2} ms", snapshot.frame.max_frame_time_ms), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Render", &format!("{:.2} ms", editor.render_system.render_time_ms), y, &rect, Color::WHITE);
        y += SECTION_SPACING;

        // Assets section
        y = self.draw_section_header(ctx, "Assets", y, &rect);
        y = self.draw_row(ctx, "Textures", &snapshot.assets.texture_count.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Tile Defs", &snapshot.assets.tile_def_count.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Sprite IDs", &snapshot.assets.sprite_id_count.to_string(), y, &rect, Color::WHITE);
        y += SECTION_SPACING;

        // Scripts section
        y = self.draw_section_header(ctx, "Scripts", y, &rect);
        y = self.draw_row(ctx, "Script IDs", &snapshot.assets.script_id_count.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Loaded", &snapshot.scripts.loaded_count.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Instances", &snapshot.scripts.instance_count.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Listeners", &snapshot.scripts.event_listener_count.to_string(), y, &rect, Color::WHITE);
        y += SECTION_SPACING;

        // ECS section
        y = self.draw_section_header(ctx, "ECS", y, &rect);
        y = self.draw_row(ctx, "Entities", &snapshot.ecs.entity_count.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Component Stores", &snapshot.ecs.component_store_count.to_string(), y, &rect, Color::WHITE);
        y += SECTION_SPACING;

        // Undo/Redo section
        y = self.draw_section_header(ctx, "Undo/Redo", y, &rect);
        y = self.draw_row(ctx, "Undo Stack", &snapshot.commands.undo_stack_size.to_string(), y, &rect, Color::WHITE);
        y = self.draw_row(ctx, "Redo Stack", &snapshot.commands.redo_stack_size.to_string(), y, &rect, Color::WHITE);
        let _ = self.draw_row(ctx, "Pending", &snapshot.commands.pending_size.to_string(), y, &rect, Color::WHITE);

        // Scrollbar
        if scroll_range > 0.0 {
            let ratio = rect.h / content_height;
            let bar_h = rect.h * ratio;
            let t = (-self.scroll_y) / scroll_range;
            let bar_x = rect.x + rect.w - SCROLLBAR_W - 2.0;
            let bar_y = rect.y + t * (rect.h - bar_h);

            ctx.draw_rectangle(bar_x, rect.y, SCROLLBAR_W, rect.h, Color::new(0.15, 0.15, 0.15, 0.6));
            ctx.draw_rectangle(bar_x, bar_y, SCROLLBAR_W, bar_h, Color::new(0.7, 0.7, 0.7, 0.9));
        }
    }
}
