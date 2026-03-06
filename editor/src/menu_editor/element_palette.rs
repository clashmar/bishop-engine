// editor/src/menu_editor/element_palette.rs
use bishop::prelude::*;

const PALETTE_ITEM_HEIGHT: f32 = 32.0;
const PALETTE_SPACING: f32 = 4.0;

/// Draggable palette of menu elements.
pub struct ElementPalette {
    scroll_y: f32,
}

impl ElementPalette {
    /// Creates a new element palette.
    pub fn new() -> Self {
        Self { scroll_y: 0.0 }
    }

    /// Renders the palette with available element types.
    pub fn draw(&mut self, ctx: &mut WgpuContext, rect: Rect, blocked: bool) {
        let mouse: Vec2 = ctx.mouse_position().into();

        if !blocked && rect.contains(mouse) {
            let (_, wheel_y) = ctx.mouse_wheel();
            self.scroll_y += wheel_y * 20.0;
        }

        let content_height = self.calculate_content_height();
        let scroll_range = (content_height - rect.h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(-scroll_range, 0.0);

        let mut y = rect.y + self.scroll_y + 8.0;

        ctx.draw_text("Elements", rect.x + 8.0, y + 14.0, 14.0, Color::GREY);
        y += 24.0;

        self.draw_palette_item(ctx, rect, &mut y, "Label", blocked);
        self.draw_palette_item(ctx, rect, &mut y, "Button", blocked);
        self.draw_palette_item(ctx, rect, &mut y, "Spacer", blocked);
        self.draw_palette_item(ctx, rect, &mut y, "Panel", blocked);
    }

    fn draw_palette_item(&self, ctx: &mut WgpuContext, rect: Rect, y: &mut f32, name: &str, blocked: bool) {
        if *y < rect.y || *y + PALETTE_ITEM_HEIGHT > rect.y + rect.h {
            *y += PALETTE_ITEM_HEIGHT + PALETTE_SPACING;
            return;
        }

        let item_rect = Rect::new(
            rect.x + 8.0,
            *y,
            rect.w - 16.0,
            PALETTE_ITEM_HEIGHT,
        );

        let mouse: Vec2 = ctx.mouse_position().into();
        let hover = item_rect.contains(mouse);

        let bg_color = if hover && !blocked {
            Color::new(0.3, 0.3, 0.35, 1.0)
        } else {
            Color::new(0.2, 0.2, 0.25, 1.0)
        };

        ctx.draw_rectangle(
            item_rect.x,
            item_rect.y,
            item_rect.w,
            item_rect.h,
            bg_color,
        );

        ctx.draw_rectangle_lines(
            item_rect.x,
            item_rect.y,
            item_rect.w,
            item_rect.h,
            1.0,
            Color::new(0.5, 0.5, 0.5, 1.0),
        );

        ctx.draw_text(
            name,
            item_rect.x + 8.0,
            item_rect.y + 20.0,
            14.0,
            Color::WHITE,
        );

        *y += PALETTE_ITEM_HEIGHT + PALETTE_SPACING;
    }

    fn calculate_content_height(&self) -> f32 {
        let header_height = 24.0;
        let item_count = 4;
        let items_height = (PALETTE_ITEM_HEIGHT + PALETTE_SPACING) * item_count as f32;
        header_height + items_height + 16.0
    }
}

impl Default for ElementPalette {
    fn default() -> Self {
        Self::new()
    }
}
