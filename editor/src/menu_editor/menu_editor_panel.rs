// editor/src/menu_editor/menu_editor_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::menu_editor::element_palette::ElementPalette;
use crate::menu_editor::properties_module::PropertiesModule;
use crate::menu_editor::canvas_module::CanvasModule;
use crate::menu_editor::menu_list_module::MenuListModule;
use crate::Editor;
use bishop::prelude::*;

pub const MENU_EDITOR_PANEL: &str = "Menu Editor";

/// Panel for visual menu composition.
pub struct MenuEditorPanel {
    menu_list_module: MenuListModule,
    element_palette: ElementPalette,
    properties_module: PropertiesModule,
    canvas_module: CanvasModule,
}

impl MenuEditorPanel {
    /// Creates a new menu editor panel.
    pub fn new() -> Self {
        Self {
            menu_list_module: MenuListModule::new(),
            element_palette: ElementPalette::new(),
            properties_module: PropertiesModule::new(),
            canvas_module: CanvasModule::new(),
        }
    }
}

impl Default for MenuEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelDefinition for MenuEditorPanel {
    fn title(&self) -> &'static str {
        MENU_EDITOR_PANEL
    }

    fn default_rect(&self, ctx: &WgpuContext) -> Rect {
        let w = 900.0;
        let h = 600.0;
        let x = (ctx.screen_width() - w) * 0.5;
        let y = (ctx.screen_height() - h) * 0.5;
        Rect::new(x, y, w, h)
    }

    fn draw(&mut self, ctx: &mut WgpuContext, rect: Rect, editor: &mut Editor, blocked: bool) {
        const LEFT_COLUMN_WIDTH: f32 = 200.0;
        const PROPERTIES_WIDTH: f32 = 250.0;
        const SPACING: f32 = 8.0;
        const MENU_LIST_HEIGHT: f32 = 180.0;

        // Left column: Menu list (top) + Element palette (bottom)
        let menu_list_rect = Rect::new(
            rect.x + SPACING,
            rect.y + SPACING,
            LEFT_COLUMN_WIDTH,
            MENU_LIST_HEIGHT,
        );

        let palette_rect = Rect::new(
            rect.x + SPACING,
            menu_list_rect.bottom() + SPACING,
            LEFT_COLUMN_WIDTH,
            rect.h - MENU_LIST_HEIGHT - SPACING * 3.0,
        );

        let properties_rect = Rect::new(
            rect.right() - PROPERTIES_WIDTH - SPACING,
            rect.y + SPACING,
            PROPERTIES_WIDTH,
            rect.h - SPACING * 2.0,
        );

        let canvas_rect = Rect::new(
            palette_rect.right() + SPACING,
            rect.y + SPACING,
            rect.w - LEFT_COLUMN_WIDTH - PROPERTIES_WIDTH - SPACING * 4.0,
            rect.h - SPACING * 2.0,
        );

        // Draw menu list background
        ctx.draw_rectangle(
            menu_list_rect.x,
            menu_list_rect.y,
            menu_list_rect.w,
            menu_list_rect.h,
            Color::new(0.15, 0.15, 0.18, 1.0),
        );

        ctx.draw_rectangle_lines(
            menu_list_rect.x,
            menu_list_rect.y,
            menu_list_rect.w,
            menu_list_rect.h,
            1.0,
            Color::new(0.4, 0.4, 0.4, 1.0),
        );

        self.menu_list_module.draw(ctx, menu_list_rect, &mut editor.menu_editor, blocked);

        // Draw element palette background
        ctx.draw_rectangle(
            palette_rect.x,
            palette_rect.y,
            palette_rect.w,
            palette_rect.h,
            Color::new(0.15, 0.15, 0.18, 1.0),
        );

        ctx.draw_rectangle_lines(
            palette_rect.x,
            palette_rect.y,
            palette_rect.w,
            palette_rect.h,
            1.0,
            Color::new(0.4, 0.4, 0.4, 1.0),
        );

        // Handle palette clicks to set pending element type
        if let Some(kind) = self.element_palette.draw(ctx, palette_rect, blocked) {
            editor.menu_editor.pending_element_type = Some(kind);
        }

        // Draw properties background
        ctx.draw_rectangle(
            properties_rect.x,
            properties_rect.y,
            properties_rect.w,
            properties_rect.h,
            Color::new(0.15, 0.15, 0.18, 1.0),
        );

        ctx.draw_rectangle_lines(
            properties_rect.x,
            properties_rect.y,
            properties_rect.w,
            properties_rect.h,
            1.0,
            Color::new(0.4, 0.4, 0.4, 1.0),
        );

        self.properties_module.draw(ctx, properties_rect, &mut editor.menu_editor, blocked);

        // Canvas
        self.canvas_module.update(ctx, canvas_rect, &mut editor.menu_editor, blocked);
        self.canvas_module.draw(ctx, canvas_rect, &editor.menu_editor);
    }
}
