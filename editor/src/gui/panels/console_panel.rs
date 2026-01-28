// editor/src/gui/panels/console_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::Editor;
use engine_core::logging::logging::LOG_HISTORY;
use engine_core::ui::text::draw_text_ui;
use engine_core::ui::widgets::*;
use macroquad::prelude::*;

const ROW_HEIGHT: f32 = 18.0;
const SCROLL_SPEED: f32 = 24.0;
const SCROLLBAR_W: f32 = 6.0;
const TOP_PADDING: f32 = 4.0;
const BOTTOM_PADDING: f32 = 8.0;
const CLEAR_BUTTON_HEIGHT: f32 = 20.0;
const HEADER_ROW_HEIGHT: f32 = 28.0;

pub struct ConsolePanel {
    scroll_y: f32,
    auto_scroll: bool,
    last_entry_count: usize,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            auto_scroll: true,
            last_entry_count: 0,
        }
    }

    fn level_color(level: log::Level) -> Color {
        match level {
            log::Level::Error => RED,
            log::Level::Warn => YELLOW,
            log::Level::Info => WHITE,
            log::Level::Debug => GRAY,
            log::Level::Trace => DARKGRAY,
        }
    }
}

pub const CONSOLE_PANEL: &str = "Console";

impl PanelDefinition for ConsolePanel {
    fn title(&self) -> &'static str {
        CONSOLE_PANEL
    }

    fn default_rect(&self) -> Rect {
        Rect::new(20., 460., 520., 200.)
    }

    fn draw(&mut self, rect: Rect, _editor: &mut Editor, blocked: bool) {
        let mouse: Vec2 = mouse_position().into();

        // Clear button centered in header row
        let btn_width = 50.0;
        let btn_y = rect.y + TOP_PADDING + (HEADER_ROW_HEIGHT - CLEAR_BUTTON_HEIGHT) / 2.0;
        let clear_btn_rect = Rect::new(
            rect.x + 6.,
            btn_y,
            btn_width,
            CLEAR_BUTTON_HEIGHT,
        );

        let clicked = gui_button(clear_btn_rect, "Clear", blocked);
        if !blocked && clicked {
            if let Ok(mut history) = LOG_HISTORY.lock() {
                history.clear();
            }
            self.scroll_y = 0.0;
        }

        // Content area below the header row
        let content_y = rect.y + TOP_PADDING + HEADER_ROW_HEIGHT;
        let content_h = rect.h - TOP_PADDING - HEADER_ROW_HEIGHT - BOTTOM_PADDING;
        let content_rect = Rect::new(rect.x, content_y, rect.w, content_h.max(0.0));

        // Scroll input
        if !blocked && content_rect.contains(mouse) {
            let (_, wheel_y) = mouse_wheel();
            if wheel_y.abs() > 0.0 {
                self.scroll_y += wheel_y * SCROLL_SPEED;
                self.auto_scroll = false;
            }
        }

        // Get log entries
        let entries: Vec<(log::Level, String)> = if let Ok(history) = LOG_HISTORY.lock() {
            history
                .entries()
                .iter()
                .map(|e| (e.level, e.message.clone()))
                .collect()
        } else {
            Vec::new()
        };

        let entry_count = entries.len();

        // Auto-scroll when new entries arrive
        if entry_count > self.last_entry_count && self.auto_scroll {
            let content_height = entry_count as f32 * ROW_HEIGHT;
            let scroll_range = (content_height - content_h).max(0.0);
            self.scroll_y = -scroll_range;
        }
        self.last_entry_count = entry_count;

        // Re-enable auto-scroll if scrolled to bottom
        let content_height = entry_count as f32 * ROW_HEIGHT;
        let scroll_range = (content_height - content_h).max(0.0);
        if scroll_range > 0.0 && self.scroll_y <= -scroll_range + 1.0 {
            self.auto_scroll = true;
        }

        // Clamp scroll
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        // Draw entries
        let usable_w = if scroll_range > 0.0 {
            content_rect.w - SCROLLBAR_W - 8.0
        } else {
            content_rect.w - 8.0
        };

        for (i, (level, message)) in entries.iter().enumerate() {
            let entry_y = content_rect.y + self.scroll_y + i as f32 * ROW_HEIGHT;

            // Skip entries outside visible area
            if entry_y + ROW_HEIGHT < content_rect.y || entry_y + ROW_HEIGHT > content_rect.y + content_rect.h {
                continue;
            }

            let color = Self::level_color(*level);

            // Truncate message if too long
            let max_chars = (usable_w / 7.0) as usize;
            let display_msg = if message.len() > max_chars {
                format!("{}...", &message[..max_chars.saturating_sub(3)])
            } else {
                message.clone()
            };

            draw_text_ui(
                &display_msg,
                content_rect.x + 6.,
                entry_y + ROW_HEIGHT * 0.75,
                14.0,
                color,
            );
        }

        // Scrollbar
        if scroll_range > 0.0 {
            let ratio = content_h / content_height;
            let bar_h = content_h * ratio;
            let t = (-self.scroll_y) / scroll_range;
            let bar_x = content_rect.x + content_rect.w - SCROLLBAR_W - 2.0;
            let bar_y = content_rect.y + t * (content_h - bar_h);

            // Track
            draw_rectangle(
                bar_x,
                content_rect.y,
                SCROLLBAR_W,
                content_h,
                Color::new(0.15, 0.15, 0.15, 0.6),
            );
            // Thumb
            draw_rectangle(
                bar_x,
                bar_y,
                SCROLLBAR_W,
                bar_h,
                Color::new(0.7, 0.7, 0.7, 0.9),
            );
        }
    }
}
