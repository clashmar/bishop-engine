// editor/src/gui/panels/console_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::Editor;
use engine_core::logging::logging::LOG_HISTORY;
use engine_core::ui::widgets::Button;
use engine_core::ui::text::*;
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

    fn level_prefix(level: log::Level) -> &'static str {
        match level {
            log::Level::Error => "[ERROR] ",
            log::Level::Warn => "[WARN]  ",
            log::Level::Info => "[INFO]  ",
            log::Level::Debug => "[DEBUG] ",
            log::Level::Trace => "[TRACE] ",
        }
    }

    /// Wraps text to fit within the given pixel width.
    fn wrap_text(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        if text.is_empty() {
            return vec![String::new()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width: f32 = 0.0;

        // Split into tokens that preserve delimiters as separate tokens
        let tokens = Self::tokenize_for_wrap(text);

        for token in tokens {
            let token_width = measure_text_ui(&token, font_size, 1.0).width;

            // If this token alone exceeds max_width, break it character by character
            if token_width > max_width {
                // First, push current line if non-empty
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_width = 0.0;
                }

                // Break the long token character by character
                for c in token.chars() {
                    let char_str = c.to_string();
                    let char_width = measure_text_ui(&char_str, font_size, 1.0).width;

                    if current_width + char_width > max_width && !current_line.is_empty() {
                        lines.push(current_line);
                        current_line = String::new();
                        current_width = 0.0;
                    }

                    current_line.push(c);
                    current_width += char_width;
                }
            } else if current_width + token_width > max_width {
                // Token doesn't fit on current line, start a new line
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                // Skip leading whitespace on new lines
                let trimmed = token.trim_start();
                current_line = trimmed.to_string();
                current_width = measure_text_ui(&current_line, font_size, 1.0).width;
            } else {
                // Token fits, append it
                current_line.push_str(&token);
                current_width += token_width;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Splits text into tokens for wrapping, keeping delimiters as break points.
    fn tokenize_for_wrap(text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            // These characters are good break points
            if c == ' ' || c == '/' || c == '\\' || c == '-' || c == '_' || c == '.' {
                if !current.is_empty() {
                    tokens.push(current);
                    current = String::new();
                }
                tokens.push(c.to_string());
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
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
            rect.x + rect.w - btn_width - 6.,
            btn_y,
            btn_width,
            CLEAR_BUTTON_HEIGHT,
        );

        let clicked = Button::new(clear_btn_rect, "Clear").blocked(blocked).show();
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

        // Calculate usable width for text
        let usable_w = content_rect.w - SCROLLBAR_W - 12.0;
        let font_size = 14.0;

        // Pre-calculate wrapped lines for all entries 
        let wrapped_entries: Vec<(log::Level, Vec<String>)> = entries
            .iter()
            .map(|(level, message)| {
                let prefix = Self::level_prefix(*level);
                let full_message = format!("{}{}", prefix, message);
                let lines = Self::wrap_text(&full_message, usable_w, font_size);
                (*level, lines)
            })
            .collect();

        // Calculate total line count for content height
        let total_lines: usize = wrapped_entries.iter().map(|(_, lines)| lines.len()).sum();
        let content_height = total_lines as f32 * ROW_HEIGHT;
        let scroll_range = (content_height - content_h).max(0.0);

        // Auto-scroll when new entries arrive
        if entry_count > self.last_entry_count && self.auto_scroll {
            self.scroll_y = -scroll_range;
        }
        self.last_entry_count = entry_count;

        // Re-enable auto-scroll if scrolled to bottom
        if scroll_range > 0.0 && self.scroll_y <= -scroll_range + 1.0 {
            self.auto_scroll = true;
        }

        // Clamp scroll
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        // Draw entries with cumulative Y tracking
        let mut cumulative_y = 0.0;

        for (level, lines) in &wrapped_entries {
            let entry_height = lines.len() as f32 * ROW_HEIGHT;
            let entry_top = content_rect.y + self.scroll_y + cumulative_y;
            let entry_bottom = entry_top + entry_height;

            // Skip entries entirely outside visible area
            if entry_bottom < content_rect.y || entry_top > content_rect.y + content_rect.h {
                cumulative_y += entry_height;
                continue;
            }

            let color = Self::level_color(*level);

            // Draw each line of the wrapped entry
            for (line_idx, line) in lines.iter().enumerate() {
                let line_y = entry_top + line_idx as f32 * ROW_HEIGHT;

                // Skip individual lines outside visible area
                if line_y < content_rect.y || line_y + ROW_HEIGHT > content_rect.y + content_rect.h {
                    continue;
                }

                draw_text_ui(
                    line,
                    content_rect.x + 6.,
                    line_y + ROW_HEIGHT * 0.75,
                    font_size,
                    color,
                );
            }

            cumulative_y += entry_height;
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
