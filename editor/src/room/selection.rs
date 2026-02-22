// editor/src/room/selection.rs
use crate::gui::panels::panel_manager::is_mouse_over_panel;
use crate::gui::modal::is_modal_open;
use crate::room::room_editor::*;
use std::collections::HashSet;
use engine_core::prelude::*;
use bishop::prelude::*;

/// Stores the original drag state before switching to copy mode.
pub(crate) struct PreCopyDragState {
    pub anchor_entity: Option<Entity>,
    pub selected_entities: HashSet<Entity>,
}

impl RoomEditor {
    /// Sets a single selected entity for the room editor, clearing any previous selection.
    pub fn set_selected_entity(&mut self, entity: Option<Entity>) {
        self.selected_entities.clear();
        if let Some(e) = entity {
            self.selected_entities.insert(e);
        }
        self.inspector.set_target(entity);
    }

    /// Adds an entity to the current selection.
    pub fn add_to_selection(&mut self, entity: Entity) {
        self.selected_entities.insert(entity);
        // Update inspector only if this is now the only selection
        if self.selected_entities.len() == 1 {
            self.inspector.set_target(Some(entity));
        } else {
            self.inspector.set_target(None);
        }
    }

    /// Clears the entire selection.
    pub fn clear_selection(&mut self) {
        self.selected_entities.clear();
        self.inspector.set_target(None);
    }

    /// Returns whether the given entity is currently selected.
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected_entities.contains(&entity)
    }

    /// Returns the single selected entity if exactly one is selected.
    pub fn single_selected_entity(&self) -> Option<Entity> {
        if self.selected_entities.len() == 1 {
            self.selected_entities.iter().next().copied()
        } else {
            None
        }
    }

    /// Selects all entities in the specified room.
    pub fn select_all_in_room(&mut self, ecs: &Ecs, room_id: RoomId) {
        self.selected_entities.clear();
        for (entity, _) in ecs.get_store::<Transform>().data.iter() {
            if can_select_entity_in_room(ecs, *entity, room_id) {
                self.selected_entities.insert(*entity);
            }
        }
        // Clear inspector since we likely have multiple selected
        if self.selected_entities.len() != 1 {
            self.inspector.set_target(None);
        } else if let Some(e) = self.selected_entities.iter().next() {
            self.inspector.set_target(Some(*e));
        }
    }

    #[inline]
    pub fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }

    pub fn is_mouse_over_ui(&self) -> bool {
        let mouse_screen: Vec2 = mouse_position().into();
        self.active_rects.iter().any(|r| r.contains(mouse_screen))
            || self.sub_mode_rect.map_or(false, |r| r.contains(mouse_screen))
            || self.inspector.is_mouse_over()
            || is_dropdown_open()
            || is_modal_open()
            || is_mouse_over_panel()
    }

    pub(crate) fn ui_was_clicked(&self) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && self.is_mouse_over_ui()
    }

    pub(crate) fn handle_mouse_cursor(&self) {
        if self.is_mouse_over_ui() {
            set_cursor_icon(CursorIcon::Default);
        } else {
            match self.mode {
                RoomEditorMode::Scene => {
                    set_cursor_icon(CursorIcon::Default);
                }
                RoomEditorMode::Tilemap => {
                    set_cursor_icon(CursorIcon::Crosshair);
                }
            }
        }
    }
}

/// Returns true if an entity can be selected in a room (is in the room).
pub fn can_select_entity_in_room(
    ecs: &Ecs,
    entity: Entity,
    room_id: RoomId,
) -> bool {
    // Make sure the entity is in the requested room
    match ecs.get_store::<CurrentRoom>().get(entity) {
        Some(CurrentRoom(id)) => *id == room_id,
        None => false,
    }
}
