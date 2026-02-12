// editor/src/room/room_editor.rs
use crate::editor_camera_controller::EditorCameraController;
use crate::tilemap::tilemap_editor::TileMapEditor;
use crate::gui::inspector::inspector::Inspector;
use crate::editor_assets::editor_assets::*;
use crate::room::drawing::*;
use crate::room::selection::PreCopyDragState;
use crate::commands::room::*;
use crate::gui::mode_selector::*;
use crate::ecs::transform::*;
use engine_core::ecs::component::{CurrentRoom, Name};
use crate::editor_global::*;
use crate::canvas::grid;
use crate::world::coord;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::assets::asset_manager::AssetManager;
use engine_core::animation::animation_system::*;
use engine_core::controls::controls::Controls;
use engine_core::rendering::render_room::*;
use engine_core::camera::game_camera::*;
use engine_core::world::world::World;
use engine_core::ecs::entity::Entity;
use engine_core::world::room::*;
use engine_core::ui::widgets::*;
use engine_core::ecs::ecs::Ecs;
use engine_core::game::game::*;
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use macroquad::prelude::*;
use once_cell::sync::Lazy;

#[derive(Clone, Copy, PartialEq, EnumIter)]
pub enum RoomEditorMode {
    Scene,
    Tilemap,
}

impl ModeInfo for RoomEditorMode {
    fn label(&self) -> &'static str {
        match self {
            RoomEditorMode::Scene => "Scene Editor: S",
            RoomEditorMode::Tilemap => "Tilemap Editor: T",
        }
    }
    fn icon(&self) -> &'static Texture2D {
        match self {
            RoomEditorMode::Scene => &ENTITY_ICON,
            RoomEditorMode::Tilemap => &GRID_ICON,
        }
    }
    fn shortcut(self) -> Option<fn() -> bool> {
        match self {
            RoomEditorMode::Scene => Some(Controls::s),
            RoomEditorMode::Tilemap => Some(Controls::t),
        }
    }
}

pub struct RoomEditor {
    pub mode: RoomEditorMode,
    pub mode_selector: ModeSelector<RoomEditorMode>,
    pub tilemap_editor: TileMapEditor,
    pub inspector: Inspector,
    pub selected_entities: HashSet<Entity>,
    pub(crate) active_rects: Vec<Rect>,
    pub(crate) show_grid: bool,
    pub(crate) drag_offset: Vec2,
    pub(crate) dragging: bool,
    /// Stores for all dragged entities.
    pub(crate) drag_start_positions: Vec<(Entity, Vec2)>,
    /// The entity that was clicked to start the selection drag.
    pub(crate) drag_anchor_entity: Option<Entity>,
    initialized: bool,
    pub create_entity_requested: bool,
    pub request_play: bool,
    pub view_preview: bool,
    pub(crate) preview_camera_id: Option<usize>,
    /// Start position of the box selection in world coordinates.
    pub(crate) box_select_start: Option<Vec2>,
    /// Whether box selection is currently active.
    pub(crate) box_select_active: bool,
    /// Whether current drag is an alt+drag copy operation.
    pub(crate) alt_copy_mode: bool,
    /// Entities created during alt+drag copy for undo command.
    pub(crate) alt_copied_entities: Vec<Entity>,
    /// Original drag state before entering copy mode (for reverting on alt release).
    pub(crate) pre_copy_drag_state: Option<PreCopyDragState>,
    /// The very first start positions when drag began (for undo command).
    pub(crate) drag_initial_start_positions: Vec<(Entity, Vec2)>,
}

impl RoomEditor {
    pub fn new() -> Self {
        let mode = RoomEditorMode::Scene;

        Self {
            mode: RoomEditorMode::Scene,
            mode_selector: ModeSelector {
                current: mode,
                options: *ALL_MODES,
            },
            tilemap_editor: TileMapEditor::new(),
            inspector: Inspector::new(),
            selected_entities: HashSet::new(),
            active_rects: Vec::new(),
            show_grid: true,
            drag_offset: Vec2::ZERO,
            dragging: false,
            drag_start_positions: Vec::new(),
            drag_anchor_entity: None,
            initialized: false,
            preview_camera_id: None,
            create_entity_requested: false,
            request_play: false,
            view_preview: false,
            box_select_start: None,
            box_select_active: false,
            alt_copy_mode: false,
            alt_copied_entities: Vec::new(),
            pre_copy_drag_state: None,
            drag_initial_start_positions: Vec::new(),
        }
    }

    pub async fn update(
        &mut self,
        camera: &mut Camera2D,
        room_id: RoomId,
        ecs: &mut Ecs,
        current_world: &mut World,
        asset_manager: &mut AssetManager,
    ) {
        let grid_size = current_world.grid_size;

        let other_bounds: Vec<(Vec2, Vec2)> = current_world.rooms
            .iter()
            .filter(|r| r.id != room_id)
            .map(|r| (r.position, r.size))
            .collect();

        // Compute exits from adjacent rooms that face toward the current room
        let adjacent_exits: Vec<(Vec2, ExitDirection)> = {
            let current_room = current_world.rooms
                .iter()
                .find(|r| r.id == room_id);

            match current_room {
                Some(target) => current_world.rooms
                    .iter()
                    .filter(|r| r.id != room_id)
                    .flat_map(|adj| adj.exits_facing_room(target, grid_size))
                    .collect(),
                None => vec![],
            }
        };

        let room = current_world.rooms
            .iter_mut()
            .find(|r| r.id == room_id)
            .expect("Could not find room in world.");

        if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
            clear_all_input_focus();
        }

        if !self.initialized {
            EditorCameraController::reset_room_editor_camera(camera, room, grid_size);
            self.initialized = true;
        }

        self.handle_mouse_cursor();

        // Click-selection
        let mouse_screen: Vec2 = mouse_position().into();

        let mut ui_was_clicked = false;

        let delta_time = get_frame_time();

        update_animation_sytem(
            ecs,
            asset_manager,
            delta_time,
            room.id
        ).await;

        match self.mode {
            RoomEditorMode::Tilemap => {
                self.tilemap_editor.update(
                    asset_manager,
                    camera,
                    room,
                    &other_bounds,
                    &adjacent_exits,
                    ecs,
                    grid_size,
                    room_id,
                ).await;
            }
            RoomEditorMode::Scene => {
                if self.ui_was_clicked() {
                    ui_was_clicked = true;
                }

                let drag_handled = self.handle_selection(
                    room.id,
                    camera,
                    ecs,
                    asset_manager,
                    mouse_screen,
                    ui_was_clicked,
                    grid_size,
                );

                if !drag_handled {
                    self.handle_keyboard_move(ecs, room.id);
                }

                // Handle batch delete when multiple entities selected
                if self.selected_entities.len() > 1
                    && Controls::delete()
                    && !input_is_focused()
                {
                    let entities: Vec<Entity> = self.selected_entities.iter().copied().collect();
                    push_command(Box::new(BatchDeleteEntitiesCmd::new(entities, room.id)));
                }

                // Copy multiple selected entities
                if Controls::copy() && self.selected_entities.len() > 1 && !input_is_focused() {
                    let entities: Vec<Entity> = self.selected_entities.iter().copied().collect();
                    copy_entities(ecs, &entities);
                }

                // Create a new entity if create was pressed
                if self.create_entity_requested && self.inspector.target.is_none() {
                    // Build the entity
                    let entity = ecs
                        .create_entity()
                        .with(Transform { position: room.position, ..Default::default() })
                        .with(CurrentRoom(room.id))
                        .with(Name(format!("Entity")))
                        .finish();

                    // Immediately select it so the inspector shows the newly-created entity
                    self.selected_entities.clear();
                    self.selected_entities.insert(entity);
                    self.create_entity_requested = false;
                }

                // If exactly one entity is selected, show the inspector
                if self.selected_entities.len() == 1 {
                    let entity = *self.selected_entities.iter().next().unwrap();
                    self.inspector.set_target(Some(entity));
                } else {
                    self.inspector.set_target(None);
                }

                // If target was cleared by inspector, sync selection
                if self.inspector.target.is_none() && self.selected_entities.len() == 1 {
                    self.selected_entities.clear();
                }
            }
        }

        self.handle_shortcuts(camera, room, grid_size, ecs);
    }

    pub async fn draw(
        &mut self,
        camera: &Camera2D,
        room_id: RoomId,
        game: &mut Game,
        render_system: &mut RenderSystem,
    ) {
        self.request_play = false; // This is very important
        self.active_rects.clear();

        let mut game_ctx = game.ctx_mut();
        let grid_size = game_ctx.cur_world.grid_size;
        let ecs = &mut game_ctx.ecs;
        let room = &mut game_ctx.cur_world.current_room_mut().unwrap();
        let asset_manager = &mut game_ctx.asset_manager;

        let tilemap = &mut room.variants[0].tilemap;

        // Panel rect for inspector and tilemap editor
        let inspector_rect = Rect::new(
            screen_width() * 0.75,
            0.0,
            screen_width() * 0.25,
            screen_height()
        );

        match self.mode {
            RoomEditorMode::Tilemap => {
                self.tilemap_editor.tilemap_panel.set_rect(inspector_rect);
                self.tilemap_editor.draw(
                    camera,
                    tilemap,
                    asset_manager,
                    ecs,
                    room_id,
                    room.position,
                    room.size,
                    grid_size,
                ).await;

                if self.show_grid {
                    set_camera(camera);
                    grid::draw_grid(camera, grid_size);
                    draw_exit_placeholders(&room.exits, room.position, grid_size);
                }
            }
            RoomEditorMode::Scene => {
                let room_camera = get_room_camera_by_id(ecs, room_id, grid_size, self.preview_camera_id);

                let render_cam = if self.view_preview && room_camera.is_some() {
                    room_camera.as_ref().map(|c| &c.camera).unwrap_or(camera)
                } else {
                    camera
                };

                self.inspector.set_rect(inspector_rect);

                if self.view_preview {
                    render_system.resize_for_camera(render_cam.zoom);
                } else {
                    render_system.resize_to_window();
                }

                // Draws everything in the room. Same implementation as the game.
                render_room(
                    ecs,
                    room,
                    asset_manager,
                    render_system,
                    render_cam,
                    0.0,
                    None,
                    grid_size,
                );

                if self.view_preview {
                    render_system.present_game();
                } else {
                    render_system.present();
                }

                if !self.view_preview {
                    set_camera(camera);

                    if self.show_grid {
                        grid::draw_grid(camera, grid_size);
                    }

                    draw_exit_placeholders(&room.exits, room.position, grid_size);
                    draw_camera_placeholders(&ecs, room_id, grid_size);
                    draw_light_placeholders(ecs, room_id, grid_size);
                    draw_glow_placeholders(ecs, asset_manager, room_id, grid_size);

                    // Highlight all selected entities and draw their overlays
                    for &selected_entity in &self.selected_entities {
                        if !is_pure_placeholder(ecs, selected_entity) {
                            highlight_selected_entity(ecs, selected_entity, asset_manager, YELLOW, grid_size);
                        }
                        self.draw_camera_viewport(camera, ecs, selected_entity, room_id);
                        draw_pivot_marker(ecs, selected_entity);
                    }

                    // Draw collider only for single selection
                    if let Some(selected_entity) = self.single_selected_entity() {
                        draw_collider(ecs, selected_entity);
                    }

                    // Draw box selection rectangle
                    if self.box_select_active {
                        if let Some(start) = self.box_select_start {
                            let mouse_world = coord::mouse_world_pos(camera);
                            draw_selection_box(start, mouse_world);
                        }
                    }
                }
            }
        }

        // Scene UI
        if !self.view_preview {
            self.draw_ui(&mut game_ctx, camera);
        }
    }

    pub fn reset(&mut self) {
        self.tilemap_editor.reset();
        self.mode = RoomEditorMode::Scene;
        self.selected_entities.clear();
        self.initialized = false;
        self.request_play = false;
        self.view_preview = false;
        self.preview_camera_id = None;
        self.box_select_start = None;
        self.box_select_active = false;
        self.drag_start_positions.clear();
        self.drag_initial_start_positions.clear();
        self.drag_anchor_entity = None;
        self.alt_copy_mode = false;
        self.alt_copied_entities.clear();
        self.pre_copy_drag_state = None;
    }

    /// Takes any pending toast message from the room/tilemap editor.
    pub fn take_pending_toast(&mut self) -> Option<&'static str> {
        self.tilemap_editor.take_pending_toast()
    }
}

/// A slice of all the modes.
static ALL_MODES: Lazy<&'static [RoomEditorMode]> = Lazy::new(|| {
    Box::leak(Box::new(
        RoomEditorMode::iter().collect::<Vec<_>>()
    ))
});
