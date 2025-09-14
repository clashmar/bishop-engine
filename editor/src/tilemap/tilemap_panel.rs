// editor/src/tilemap/tilemap_panel.rs
use crate::tilemap::{
    tile_palette::{TilePalette, TilePaletteUi, TilePaletteUiMode},
};
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::world_ecs::WorldEcs
};
use macroquad::prelude::*;
use engine_core::ui::widgets::*;

const INSET: f32 = 10.0;
const BTN_HEIGHT: f32 = 30.0;

/// The panel that lives on the right‑hand side of the tilemap editor window
pub struct TilemapPanel {
    /// Geometry of the panel
    pub rect: Rect,
    /// Module responsible for tile creation/selection
    pub palette: TilePalette,
    /// Rectangles that were drawn this frame and are therefore active.
    active_rects: Vec<Rect>,
}

impl TilemapPanel {
    /// Create a fresh panel with all modules
    pub fn new() -> Self {
        let palette = TilePalette::new();

        // TODO: Add other modules
        
        Self {
            rect: Rect::new(0., 0., 0., 0.),
            palette,
            active_rects: Vec::new(),
        }
    }

    /// Called by the editor each frame to place the panel
    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    /// Render the panel and any visible sub‑modules
    pub fn draw(
        &mut self,
        asset_manager: &mut AssetManager,
        world_ecs: &WorldEcs,
    ) {
        self.active_rects.clear();

        const PADDING: f32 = 20.0;
        const SPACING: f32 = 10.0;   

        // Create button
        let create_label = "Create Tile";
        let create_width = measure_text(create_label, None, 20, 1.0).width + PADDING;
        let create_start = screen_width() - INSET - create_width;
        let create_rect = self.register_rect(Rect::new(create_start, INSET, create_width, BTN_HEIGHT));

        if gui_button(create_rect, create_label) {
            if self.palette.ui.open && self.palette.ui.mode == TilePaletteUiMode::Create {
                self.palette.ui.open = false; // hide dialog
            } else {
                self.palette.ui = TilePaletteUi::default(); // reset fields
                self.palette.ui.open = true;
                self.palette.ui.mode = TilePaletteUiMode::Create;
            }
        }

        // Edit button appears only when there is a selected palette tile
        if !self.palette.entries.is_empty() {
            let edit_label = "Edit";
            let edit_width = measure_text(edit_label, None, 20, 1.0).width + PADDING;
            let edit_start = screen_width() - INSET - SPACING - create_width - edit_width;
            let edit_rect = self.register_rect(Rect::new(edit_start, INSET, edit_width, BTN_HEIGHT));

            if gui_button(edit_rect, edit_label) {
                self.palette.ui.mode = TilePaletteUiMode::Edit;
                self.palette.ui.edit_index = self.palette.selected_index;
                self.palette.ui.edit_initialized = true;
                self.palette.ui.open = true;
            }
        }

        // Compute the top offset for the panel
        let top_offset = create_rect.y + BTN_HEIGHT + INSET;
        
        // Reduce the height so the panel still fits
        let inner = Rect::new(
            self.rect.x,
            top_offset,
            self.rect.w - INSET,
            self.rect.h - (top_offset - self.rect.y) - INSET,
        );

        // Background
        draw_rectangle(
            inner.x,
            inner.y,
            inner.w,
            inner.h,
            Color::new(0., 0., 0., 0.6),
        );

        // Outline
        draw_rectangle_lines(inner.x, inner.y, inner.w, inner.h, 2., WHITE);

        // Layout the modules vertically
        let mut y = inner.y + 10.0;

        // Palette
        self.palette.set_columns_for_width(inner.w - 20.0);
        let height = self.palette.height();
        let palette_rect = Rect::new(inner.x + 10.0, y, inner.w, height);
        self.palette.draw(palette_rect, asset_manager, world_ecs);

        y += height + 10.0; // Create gap for future modules
    }

    pub fn handle_click(&mut self, mouse_pos: Vec2, rect: Rect) -> bool {
        let top_offset = rect.y + INSET + BTN_HEIGHT + INSET;
        let inner = Rect::new(
            rect.x + INSET,
            top_offset,
            rect.w - 2.0 * INSET,
            rect.h - (top_offset - rect.y) - INSET,
        );

        self.palette.handle_click(mouse_pos, inner)
     }

    #[inline]
    fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }
}