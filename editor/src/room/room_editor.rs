// editor/src/room/room_editor.rs
use crate::{
    camera_controller::CameraController, 
    canvas::grid, 
    gui::inspector::inspector_panel::InspectorPanel, 
    tilemap::tilemap_editor::TileMapEditor, 
    world::coord,
    gui::gui_constants::*,
};
use engine_core::{
    assets::asset_manager::AssetManager, 
    ecs::{component::{CurrentRoom, Position}, 
    entity::Entity, 
    world_ecs::WorldEcs}, 
    rendering::render_entities::*, 
    ui::widgets::*, 
    world::room::Room,
};
use macroquad::prelude::*;

pub enum RoomEditorMode {
    Tilemap,
    Scene,
}

pub struct RoomEditor {
    pub mode: RoomEditorMode,
    pub tilemap_editor: TileMapEditor,
    pub inspector: InspectorPanel,
    selected_entity: Option<Entity>,
    show_grid: bool,
    drag_offset: Vec2,
    dragging: bool,
    initialized: bool, 
    create_entity_requested: bool,
    pub request_play: bool,
}

impl RoomEditor {
    pub fn new() -> Self {
        Self {
            mode: RoomEditorMode::Scene,
            tilemap_editor: TileMapEditor::new(),
            inspector: InspectorPanel::new(),
            selected_entity: None,
            show_grid: true,
            drag_offset: Vec2::ZERO,
            dragging: false,
            initialized: false,
            create_entity_requested: false,
            request_play: false,
        }
    }

    /// Returns `true` if user wants to exit back to world view.  
    pub async fn update(
        &mut self, 
        camera: &mut Camera2D,
        room: &mut Room,
        other_bounds: &Vec<(Vec2, Vec2)>,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) -> bool {
        if is_key_pressed(KeyCode::Escape) {
            self.tilemap_editor.reset();
            self.reset();
            return true;
        }

        if !self.initialized {
            CameraController::reset_room_camera(camera, room);
            self.initialized = true;
        }

        // Click‑selection
        let mouse_screen: Vec2 = mouse_position().into();

        let mut ui_was_clicked = false;

        match self.mode {
            RoomEditorMode::Tilemap => {
                self.tilemap_editor.update(
                    camera,
                    room, 
                    &other_bounds, 
                    world_ecs, 
                    asset_manager
                ).await;
            }
            RoomEditorMode::Scene => {
                if self.inspector.was_clicked(mouse_screen) {
                    ui_was_clicked = true;
                }

                if !ui_was_clicked && is_mouse_button_pressed(MouseButton::Left) && !self.dragging {
                    self.selected_entity = None;
                    for (entity, position) in world_ecs.get_store::<Position>().data.iter() {
                        let room_position = position.position - room.position;
                        let screen = coord::world_to_screen(camera, room_position);
                        let hit = Rect::new(screen.x - 10.0, screen.y - 10.0, 20.0, 20.0);
                        if hit.contains(mouse_screen) {
                            self.selected_entity = Some(*entity);
                            let mouse_world = coord::mouse_world_pos(camera);
                            self.drag_offset = room_position - mouse_world;
                            self.dragging = true;
                            break;
                        }
                    }
                }

                // Dragging
                if self.dragging {
                    if let Some(entity) = self.selected_entity {
                        if let Some(position) = world_ecs.get_store_mut::<Position>().get_mut(entity) {
                            let mouse_world = coord::mouse_world_pos(camera);
                            position.position = mouse_world + room.position + self.drag_offset;
                        }
                    }
                    if is_mouse_button_released(MouseButton::Left) {
                        self.dragging = false;
                    }
                }

                // Create a new entity if create was pressed
                if self.create_entity_requested && self.inspector.target.is_none() {
                    // Build the entity
                    let entity = world_ecs
                        .create_entity()
                        .with(Position { position: room.position })
                        .with(CurrentRoom(room.id))
                        .finish();

                    // Immediately select it so the inspector shows the newly‑created entity
                    self.selected_entity = Some(entity);
                    self.inspector.set_target(Some(entity));
                    self.create_entity_requested = false;
                }
            }
        }

        if is_key_pressed(KeyCode::Tab) {
            self.mode = match self.mode {
                RoomEditorMode::Tilemap => RoomEditorMode::Scene,
                RoomEditorMode::Scene => RoomEditorMode::Tilemap,
            };
        }

        if is_key_pressed(KeyCode::G) {
            self.show_grid = !self.show_grid;
        }

        if is_key_pressed(KeyCode::R) {
            CameraController::reset_room_camera(camera, room);
        }

        false
    }

    pub fn draw(
        &mut self, 
        camera: &Camera2D,
        room: &mut Room,
        world_ecs: &mut WorldEcs, 
        asset_manager: &mut AssetManager
    ) {
        self.request_play = false; // This is very important

        let tilemap = &mut room.variants[0].tilemap;
        let exits = &room.exits;

        // Panel rect for inspector and tilemap editor
        let inspector_rect = Rect::new(
            screen_width() * 0.75, 
            0.0, 
            screen_width() * 0.25, 
            screen_height()
        );

        match self.mode {
            RoomEditorMode::Tilemap => {
                self.tilemap_editor.panel.set_rect(inspector_rect);
                self.tilemap_editor.draw(
                    camera, 
                    tilemap, 
                    exits, 
                    world_ecs,
                    asset_manager,
                );
            }
            RoomEditorMode::Scene => {
                self.inspector.set_rect(inspector_rect);

                tilemap.draw(camera, exits, world_ecs, asset_manager);

                draw_entities(world_ecs, room, asset_manager);

                if let Some(sel) = self.selected_entity {
                    highlight_selected_entity(world_ecs, room, sel);
                }
                
                set_default_camera();
                
                // If an entity is selected, forward it to the inspector.
                if let Some(entity) = self.selected_entity {
                    self.inspector.set_target(Some(entity));
                } else {
                    self.inspector.set_target(None); // clears the panel
                }
                
                self.create_entity_requested = self.inspector.draw(asset_manager, world_ecs);

                if self.inspector.target.is_none() {
                    self.selected_entity = None;
                }
            }
        }

        if self.show_grid {
            set_camera(camera);
            grid::draw_grid(camera);
        }

        set_default_camera();

        // Play‑test button
        if matches!(self.mode, RoomEditorMode::Scene) {
            // Build button
            let play_label = "Play";
            let play_width = measure_text(play_label, None, 20, 1.0).width + PADDING;
            let play_x = (screen_width() - play_width) / 2.0;
            let play_rect = Rect::new(play_x, INSET, play_width, BTN_HEIGHT);

            if gui_button(play_rect, play_label) {
                self.request_play = true;
            }
        }
        
        self.draw_coordinates(camera, room);
    }

    pub fn reset(&mut self) {
        self.mode = RoomEditorMode::Scene;
        self.selected_entity = None;
        self.initialized = false;
        self.request_play = false;
    }
}