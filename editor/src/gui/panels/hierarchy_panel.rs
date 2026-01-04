// editor/src/gui/panels/hierarchy_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::gui::gui_constants::*;
use crate::ecs::component::Name;
use crate::Editor;
use crate::room::room_editor::RoomEditor;
use engine_core::ecs::component::{ComponentStore, CurrentRoom, Global};
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
pub const HIERARCHY_PANEL: &'static str = "Hierarchy";

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
            editor.game.ecs.create_entity()
                .with(Global::default())
                .with(Name(format!("Global Entity")));
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

        // Grab references to parts needed for each node
        let name_store = editor.game.ecs.get_store::<Name>();
        let room_editor = &mut editor.room_editor;

        for entity in global_entities {
            draw_entity_node(
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
                room_editor,
                name_store,
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
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
                room_editor,
                name_store,
            );
        }
    }
}

fn draw_entity_node(
    entity: Entity,
    depth: usize,
    panel_rect: Rect,
    y: &mut f32,
    expanded: &mut HashSet<Entity>,
    room_editor: &mut RoomEditor,
    name_store: &ComponentStore<Name>,
) {
    let indent = depth as f32 * 14.0;

    let row_rect = Rect::new(
        panel_rect.x + 6. + indent,
        *y,
        panel_rect.w - 12. - indent,
        22.,
    );

    let name = match name_store.get(entity) {
        Some(name) => name.to_string(),
        None => entity.to_string()
    };

    let label = format!("{}", name);

    // Highlight if selected
    let is_selected = room_editor.selected_entity == Some(entity);

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
        room_editor.set_selected_entity(Some(entity))
    }

    *y += 22.;
}
