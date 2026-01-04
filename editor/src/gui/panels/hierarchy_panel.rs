// editor/src/gui/panels/hierarchy_panel.rs
use crate::gui::panels::generic_panel::PanelDefinition;
use crate::gui::gui_constants::*;
use crate::ecs::component::Name;
use crate::ecs::entity::*;
use crate::ecs::ecs::Ecs;
use crate::Editor;
use crate::room::room_editor::RoomEditor;
use engine_core::ui::text::draw_text_ui;
use engine_core::ecs::entity::Entity;
use engine_core::ecs::component::*;
use engine_core::ui::widgets::*;
use std::collections::HashSet;
use macroquad::prelude::*;

pub struct HierarchyPanel {
    expanded: HashSet<Entity>,
    dragging: Option<Entity>,
    drag_offset: Vec2,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            expanded: HashSet::new(),
            dragging: None,
            drag_offset: Vec2::ZERO,
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

        let global_entities: Vec<Entity> = {
            let store = editor.game.ecs.get_store::<Global>();
            let all_globals: Vec<Entity> = store.data.keys().copied().collect();
            get_root_entities(&editor.game.ecs, &all_globals)
        };

        // Grab references to parts needed for each tree
        let room_editor = &mut editor.room_editor;
        let ecs = &mut editor.game.ecs;

        for entity in global_entities {
            draw_entity_tree(
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
                &mut self.dragging,
                &mut self.drag_offset,
                room_editor,
                ecs,
            );
        }

        y += 10.;

        // Room entities
        draw_text_ui("Room", rect.x + 6., y, 14., GRAY);
        y += 18.;

        let room_entities: Vec<Entity> = {
            let store = ecs.get_store::<CurrentRoom>();
            let all_room: Vec<Entity> = store.data.keys().copied().collect();
            get_root_entities(ecs, &all_room)
        };

        for entity in room_entities {
            draw_entity_tree(
                entity,
                0,
                rect,
                &mut y,
                &mut self.expanded,
                &mut self.dragging,
                &mut self.drag_offset,
                room_editor,
                ecs,
            );
        }

        // Handle dragging
        if let Some(dragged) = self.dragging {
            let mouse_pos = mouse_position();
            
            // Get name for ghost
            let entity_name = get_entity_name(&editor.game.ecs, dragged);
            
            draw_rectangle(
                mouse_pos.0 - self.drag_offset.x,
                mouse_pos.1 - self.drag_offset.y,
                150.0,
                22.0,
                Color::new(0.3, 0.5, 0.7, 0.5),
            );
            
            draw_text_ui(
                &entity_name,
                mouse_pos.0 - self.drag_offset.x + 4.0,
                mouse_pos.1 - self.drag_offset.y + 16.0,
                14.0,
                WHITE,
            );

            // Release drag
            if is_mouse_button_released(MouseButton::Left) {
                self.dragging = None;
            }
        }
    }
}

fn get_entity_name(ecs: &Ecs, entity: Entity) -> String {
    ecs.get::<Name>(entity)
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("{:?}", entity))
}

fn draw_entity_tree(
    entity: Entity,
    depth: usize,
    panel_rect: Rect,
    y: &mut f32,
    expanded: &mut HashSet<Entity>,
    dragging: &mut Option<Entity>,
    drag_offset: &mut Vec2,
    room_editor: &mut RoomEditor,
    ecs: &mut Ecs,
) {
    let indent = depth as f32 * 16.0;
    let has_children = has_children(ecs, entity);
    let is_expanded = expanded.contains(&entity);

    // Expand/collapse button area
    let expand_btn_rect = Rect::new(
        panel_rect.x + 6. + indent,
        *y,
        14.,
        22.,
    );

    let row_rect = Rect::new(
        panel_rect.x + 6. + indent + (if has_children { 16.0 } else { 0.0 }),
        *y,
        panel_rect.w - 12. - indent - (if has_children { 16.0 } else { 0.0 }),
        22.,
    );

    let entity_name = get_entity_name(ecs, entity);

    // Highlight if selected
    let is_selected = room_editor.selected_entity == Some(entity);

    if is_selected {
        draw_rectangle(
            panel_rect.x + 6. + indent,
            row_rect.y,
            panel_rect.w - 12. - indent,
            row_rect.h,
            Color::new(0.25, 0.45, 0.85, 0.35),
        );
    }

    // Draw expand/collapse buttons
    if has_children {
        let symbol = if is_expanded { "-" } else { "+" };
        if gui_button_plain_hover(expand_btn_rect, symbol, WHITE, GRAY, false) {
            if is_expanded {
                expanded.remove(&entity);
            } else {
                expanded.insert(entity);
            }
        }
    }

    let mouse: Vec2 = mouse_position().into();
    let mouse_over = row_rect.contains(mouse);

    // Handle click for selection (do this first)
    if mouse_over && is_mouse_button_pressed(MouseButton::Left) && dragging.is_none() {
        room_editor.set_selected_entity(Some(entity));
    }

    // Start drag 
    if mouse_over && is_mouse_button_down(MouseButton::Left) && dragging.is_none() {
        *dragging = Some(entity);
        *drag_offset = Vec2::new(mouse.x - row_rect.x, mouse.y - row_rect.y);
    }

    // Drop target highlight
    if let Some(dragged_entity) = *dragging {
        if dragged_entity != entity && mouse_over && !is_ancestor(ecs, dragged_entity, entity) {
            draw_rectangle(
                row_rect.x,
                row_rect.y,
                row_rect.w,
                row_rect.h,
                Color::new(0.3, 0.7, 0.3, 0.3),
            );

            // Drop to make parent
            if is_mouse_button_released(MouseButton::Left) {
                set_parent(ecs, dragged_entity, entity);
                expanded.insert(entity);
                *dragging = None;
            }
        }
    }

    // Draw entity name
    draw_text_ui(&entity_name, row_rect.x + 4.0, row_rect.y + 16.0, 14.0, WHITE);

    *y += 22.;

    // Draw children recursively if expanded
    if is_expanded && has_children {
        let children = get_children(ecs, entity);
        for child in children {
            draw_entity_tree(
                child,
                depth + 1,
                panel_rect,
                y,
                expanded,
                dragging,
                drag_offset,
                room_editor,
                ecs,
            );
        }
    }

    // Drag outside panel to unparent
    if let Some(dragged_entity) = *dragging {
        if dragged_entity == entity {
            let mouse: Vec2 = mouse_position().into();
            let outside_panel = !panel_rect.contains(mouse);
            
            if outside_panel && is_mouse_button_released(MouseButton::Left) {
                remove_parent(ecs, dragged_entity);
                *dragging = None;
            }
        }
    }
}