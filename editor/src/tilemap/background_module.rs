// editor/src/tilemap/background_module.rs
use engine_core::{
    tiles::tilemap::TileMap, 
    ui::widgets::*
};
use macroquad::prelude::*;

// TODO: Add more complex backgrounds
/// Responsible for editing the background of a tilemap. 
pub struct BackgroundModule {
    pub r_id: WidgetId,
    pub g_id: WidgetId,
    pub b_id: WidgetId,
    pub a_id: WidgetId,
}

impl BackgroundModule {
    pub fn new() -> Self {
        Self {
            r_id: WidgetId::default(),
            g_id: WidgetId::default(),
            b_id: WidgetId::default(),
            a_id: WidgetId::default(),
        }
    }

    pub fn draw(&mut self, rect: Rect, map: &mut TileMap) {
        // Title
        draw_text("Background", rect.x, rect.y + 18.0, 18.0, WHITE);

        let mut r = map.background.r * 255.0;
        let mut g = map.background.g * 255.0;
        let mut b = map.background.b * 255.0;
        let mut a = map.background.a * 255.0;

        // Determine the width of a three‑digit number.
        // 255 is the widest possible value for an 8‑bit channel
        let sample = "255";
        let num_width = measure_text(sample, None, 20, 1.0).width;

        // Add padding so the cursor isn’t glued to the edge.
        let field_w = num_width + 13.0;
        let field_h = 30.0;
        let spacing = 5.0;

        // Position the four numeric inputs.
        let mut x = rect.x + 10.0;
        let y = rect.y + 30.0;

        r = gui_input_number(self.r_id, Rect::new(x, y, field_w, field_h), r);
        x += field_w + spacing;
        g = gui_input_number(self.g_id,Rect::new(x, y, field_w, field_h), g);
        x += field_w + spacing;
        b = gui_input_number(self.b_id,Rect::new(x, y, field_w, field_h), b);
        x += field_w + spacing;
        a = gui_input_number(self.a_id,Rect::new(x, y, field_w, field_h), a);
        x += field_w + spacing;

        // Clamp to a valid range (0‑255) and push the colour back
        r = r.clamp(0.0, 255.0);
        g = g.clamp(0.0, 255.0);
        b = b.clamp(0.0, 255.0);
        a = a.clamp(0.0, 255.0);

        map.background = Color::new(
            r / 255.0,
            g / 255.0,
            b / 255.0,
            a / 255.0,
        );

        map.background = Color::new(r / 255.0, g / 255.0, b / 255.0, a / 255.0);

        // Preview square
        let preview_sz = field_h; // same height as the input fields
        let preview_rect = Rect::new(x, y, preview_sz, preview_sz);
        draw_rectangle(preview_rect.x, preview_rect.y, preview_rect.w, preview_rect.h, map.background);
        draw_rectangle_lines(preview_rect.x, preview_rect.y, preview_rect.w, preview_rect.h, 2.0, WHITE);
    }
}