// editor/src/gui/panels/hierarchy_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::gui::gui_constants::*;
use crate::Editor;
use engine_core::ecs::component::{CurrentRoom, Global};
use engine_core::ui::text::draw_text_ui;
use engine_core::ecs::entity::Entity;
use engine_core::ui::widgets::*;
use std::collections::HashSet;
use macroquad::prelude::*;

pub struct HierarchyPanel {
    expanded: HashSet<Entity>,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            expanded: HashSet::new(),
        }
    }
}

/// Title/id of the panel.
pub const HIERARCHY_PANEL: &'static str = "Heirarchy";

impl PanelDefinition for HierarchyPanel {
    fn title(&self) -> &'static str {
        HIERARCHY_PANEL
    }

    fn default_rect(&self) -> Rect {
        Rect::new(20., 60., 260., 400.)
    }

    fn draw(&mut self, rect: Rect, editor: &mut Editor) {
        let mut y = rect.y + 6.;

        // Add global entity
        let add_rect = Rect::new(rect.x + 6., y, rect.w - 12., 26.);
        if gui_button(add_rect, "+ Global", false) {
            editor.game.ecs.create_entity().with(Global::default());
        }

        y += add_rect.h + SPACING * 2.;

        // Global entities
        draw_text_ui("Global", rect.x + 6., y, 14., GRAY);
        y += 18.;

        // Handle globals
        let global_entities: Vec<Entity> = {
            let store = editor.game.ecs.get_store::<Global>();
            store.data.keys().copied().collect()
        };

        for entity in global_entities {
            draw_entity_node(
                editor,
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
            );
        }

        y += 10.;

        // Room entities
        draw_text_ui("Room", rect.x + 6., y, 14., GRAY);
        y += 18.;

        // Handle room entities
        let room_entities: Vec<Entity> = {
            let ctx = editor.game.ctx();
            let store = ctx.ecs.get_store::<CurrentRoom>();
            store.data.keys().copied().collect()
        };

        for entity in room_entities {
            draw_entity_node(
                editor,
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
            );
        }
    }
}

fn draw_entity_node(
    editor: &mut Editor,
    entity: Entity,
    depth: usize,
    panel_rect: Rect,
    y: &mut f32,
    expanded: &mut HashSet<Entity>,
) {
    let indent = depth as f32 * 14.0;

    let row_rect = Rect::new(
        panel_rect.x + 6. + indent,
        *y,
        panel_rect.w - 12. - indent,
        22.,
    );

    let label = format!("{:?}", entity);

    // Highlight if selected
    let is_selected = editor.room_editor.selected_entity == Some(entity);

    if is_selected {
        draw_rectangle(
            row_rect.x,
            row_rect.y,
            row_rect.w,
            row_rect.h,
            Color::new(0.25, 0.45, 0.85, 0.35),
        );
    }

    if gui_button_plain_hover(row_rect, &label, WHITE, GRAY, false) {
        editor.room_editor.set_selected_entity(Some(entity))
    }

    *y += 22.;
}
