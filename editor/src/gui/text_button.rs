use engine_core::ui::text::draw_text_ui;
use macroquad::prelude::*;

pub struct TextButton {
    pub rect: Rect,
    pub label: String,
    pub background_color: Color,
    pub text_color: Color,
    pub font_size: f32,
}

impl TextButton {
    pub fn draw(&self, is_hovered: bool) {
    let bg = if is_hovered {
        Self::brighten(self.background_color, 1.2)
    } else {
        self.background_color
    };

    draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, bg);

    let text_dims = measure_text(&self.label, None, self.font_size as u16, 1.0);

    let text_x = self.rect.x + (self.rect.w - text_dims.width) / 2.0;

    let manual_offset = self.rect.h * 0.75;
    let text_y = self.rect.y + (self.rect.h / 2.0)
        + (text_dims.height / 2.0)
        - text_dims.offset_y
        + manual_offset;

    draw_text_ui(
        &self.label,
        text_x,
        text_y,
        self.font_size,
        self.text_color,
    );
}

    fn brighten(color: Color, factor: f32) -> Color {
        Color {
            r: (color.r * factor).min(1.0),
            g: (color.g * factor).min(1.0),
            b: (color.b * factor).min(1.0),
            a: color.a,
        }
    }

    pub fn is_clicked(&self, mouse_pos: Vec2) -> bool {
        self.rect.contains(mouse_pos) && is_mouse_button_pressed(MouseButton::Left)
    }

    pub fn is_hovered(&self, mouse_pos: Vec2) -> bool {
        self.rect.contains(mouse_pos)
    }
}