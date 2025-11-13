// editor/src/gui/mode_selector.rs
use engine_core::ui::text::*;
use macroquad::prelude::*;
use crate::gui::gui_constants::MENU_PANEL_HEIGHT;

/// A trait that each editor’s mode enum must implement.
pub trait ModeInfo {
    /// Human‑readable label (used for tool‑tips etc).
    fn label(&self) -> &'static str;
    /// The texture that represents the mode.
    fn icon(&self) -> &'static Texture2D;
    /// Keyboard shortcut for the mode.
    fn shortcut(self) -> Option<fn() -> bool>;
}

/// The UI component.
pub struct ModeSelector <M: ModeInfo + Copy + PartialEq + 'static> {
    /// Currently active mode.
    pub current: M,
    /// All possible modes (order defines layout).
    pub options: &'static [M],
}

impl<M: ModeInfo + Copy + PartialEq> ModeSelector<M> {
    /// Returns the total Rect drawn by the module and `true` if the mode changed.
    pub fn draw(&mut self) -> (Rect, bool) {
        let mut changed = false;
        const PADDING: f32 = 8.0;
        let icon_size = MENU_PANEL_HEIGHT - 2.0 * PADDING;

        let total_width = self.options.len() as f32 * (icon_size + PADDING) - PADDING;
        let start_x = (screen_width() - total_width) / 2.0;

        // The rect to return to callers
        let total_rect = Rect::new(start_x, PADDING - 2.0, total_width, MENU_PANEL_HEIGHT);

        // Layout the icons left to right
        for (i, mode) in self.options.iter().enumerate() {
            let x = start_x + i as f32 * (icon_size + PADDING);
            let rect = Rect::new(x, PADDING, icon_size, icon_size);

            // Highlight the active mode
            if *mode == self.current {
                draw_rectangle_lines(
                    rect.x - 2.0, rect.y - 2.0,
                    rect.w + 4.0, rect.h + 4.0,
                    2.0, 
                    YELLOW
                );
            }

            // Click handling
            if is_mouse_button_pressed(MouseButton::Left) 
                && rect.contains(mouse_position().into()) 
            {
                if *mode != self.current {
                    self.current = *mode;
                    changed = true;
                }
            }

            // Draw icon
            draw_texture_ex(
                &mode.icon(),
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );

            // Tooltip
            if rect.contains(mouse_position().into()) {
                let tip = mode.label();
                let tip_size = measure_text_ui(tip, 16.0, 1.0);

                let tip_rect = Rect::new(
                    rect.x,
                    rect.y + rect.h + 4.0,
                    tip_size.width + 8.0,
                    20.0,
                );

                draw_rectangle(
                    tip_rect.x, 
                    tip_rect.y, 
                    tip_rect.w, 
                    tip_rect.h,
                    Color::new(0.0, 0.0, 0.0, 0.8)
                );

                draw_text_ui(
                    tip, 
                    tip_rect.x + 4.0, 
                    tip_rect.y + 15.0, 
                    16.0, 
                    WHITE
                );
            }
        }
        (total_rect, changed)
    }
}
