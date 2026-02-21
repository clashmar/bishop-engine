// editor/src/room/shortcuts.rs
use crate::editor_camera_controller::EditorCameraController;
use crate::gui::panels::hierarchy_panel::HIERARCHY_PANEL;
use crate::editor_global::with_panel_manager;
use crate::gui::mode_selector::ModeInfo;
use crate::editor_global::push_command;
use crate::room::room_editor::*;
use crate::commands::room::*;
use engine_core::prelude::*;
use strum::IntoEnumIterator;
use bishop::prelude::*;

impl RoomEditor {
    pub(crate) fn handle_shortcuts(
        &mut self,
        camera: &mut Camera2D,
        room: &Room,
        grid_size: f32,
        ecs: &Ecs,
    ) {
        if input_is_focused() {
            return;
        }

        // Shortcuts for both tilemap and scene
        if Controls::g() {
            self.show_grid = !self.show_grid;
        }

        if Controls::r() {
            EditorCameraController::reset_room_editor_camera(camera, room, grid_size);
        }

        for mode in RoomEditorMode::iter() {
            if let Some(is_pressed) = mode.shortcut() {
                if is_pressed() {
                    self.mode = mode;
                    self.mode_selector.current = mode;
                    break;
                }
            }
        }

        match self.mode {
            RoomEditorMode::Tilemap => {

            }
            RoomEditorMode::Scene => {
                if Controls::v() {
                    self.view_preview = !self.view_preview;
                    if self.view_preview {
                        // If a single camera is selected, use it
                        let camera_id = self.single_selected_entity()
                            .filter(|e| ecs.has::<RoomCamera>(*e))
                            .map(|e| e.0);

                        if camera_id.is_some() {
                            self.preview_camera_id = camera_id;
                        } else {
                            let first_camera = get_next_room_camera(ecs, room.id, grid_size, None);
                            self.preview_camera_id = first_camera.map(|c| c.id);
                        }
                    } else {
                        self.preview_camera_id = None;
                    }
                }

                if self.view_preview && Controls::tab() {
                    let next_camera = get_next_room_camera(ecs, room.id, grid_size, self.preview_camera_id);
                    self.preview_camera_id = next_camera.map(|c| c.id);
                }

                if Controls::paste() {
                    push_command(Box::new(PasteEntityCmd::new(room.id)));
                }

                if Controls::h() {
                    with_panel_manager(|panel_manager| {
                        panel_manager.toggle(HIERARCHY_PANEL);
                    });
                }

                // Select all entities in room
                if Controls::select_all() {
                    self.select_all_in_room(ecs, room.id);
                }

                // Duplicate selected entities
                if Controls::duplicate() && !self.selected_entities.is_empty() {
                    let entities: Vec<Entity> = self.selected_entities.iter().copied().collect();
                    push_command(Box::new(DuplicateEntitiesCmd::new(entities, room.id)));
                }
            }
        }
    }
}
