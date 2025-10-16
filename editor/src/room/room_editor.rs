// editor/src/room/room_editor.rs
use crate::{
    canvas::grid, 
    commands::entity_commands::{MoveEntityCmd, PasteEntityCmd}, 
    controls::controls::Controls, 
    editor_camera_controller::*, 
    global::push_command, 
    gui::{
        gui_constants::*, 
        inspector::inspector_panel::InspectorPanel
    }, 
    room::room_editor_rendering::*, 
    tilemap::tilemap_editor::TileMapEditor, 
    world::coord
};
use engine_core::{
    animation::animation_system::update_animation_sytem, assets::asset_manager::AssetManager, ecs::{
        component::{CurrentRoom, Position, RoomCamera}, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, lighting::{light::Light, light_system::LightSystem}, rendering::render_room::*, tiles::tile::TileSprite, ui::widgets::*, world::room::Room
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
    pub selected_entity: Option<Entity>,
    pub light_system: LightSystem,
    show_grid: bool,
    drag_offset: Vec2,
    dragging: bool,
    drag_start_position: Option<Vec2>,
    initialized: bool, 
    create_entity_requested: bool,
    pub request_play: bool,
    pub view_preview: bool,
}

impl RoomEditor {
    pub fn new() -> Self {
        Self {
            mode: RoomEditorMode::Scene,
            tilemap_editor: TileMapEditor::new(),
            inspector: InspectorPanel::new(),
            selected_entity: None,
            light_system: LightSystem::new(),
            show_grid: true,
            drag_offset: Vec2::ZERO,
            dragging: false,
            drag_start_position: None,
            initialized: false,
            create_entity_requested: false,
            request_play: false,
            view_preview: false,
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
        if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
            clear_all_text_focus();
        }

        if is_key_pressed(KeyCode::Escape) && !input_is_focused() {
            self.tilemap_editor.reset();
            self.reset();
            return true;
        }

        if !self.initialized {
            EditorCameraController::reset_editor_camera(camera, room);
            self.initialized = true;
        }
        
        if Controls::paste() {
            push_command(Box::new(PasteEntityCmd::new()));
        }

        // Click‑selection
        let mouse_screen: Vec2 = mouse_position().into();

        let mut ui_was_clicked = false;

        let delta_time = get_frame_time();
        
        update_animation_sytem(
            world_ecs,
            asset_manager,
            delta_time, 
            room.id
        ).await;

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
                if self.inspector.was_clicked() {
                    ui_was_clicked = true;
                }

                // Detect dragging
                if !ui_was_clicked && is_mouse_button_pressed(MouseButton::Left) && !self.dragging {

                    self.selected_entity = None;
                    for (entity, pos) in world_ecs.get_store::<Position>().data.iter() {
                        // Filter out tiles etc
                        if !can_drag_entity(world_ecs, *entity) {
                            continue;
                        }
                        let hitbox = entity_hitbox(
                            *entity,
                            pos.position,
                            camera,
                            world_ecs,
                            asset_manager,
                        );

                        if hitbox.contains(mouse_screen) {
                            self.selected_entity = Some(*entity);
                            let mouse_world = coord::mouse_world_pos(camera);
                            self.drag_offset = pos.position - mouse_world;
                            self.dragging = true;
                            self.drag_start_position = Some(pos.position);
                            break;
                        }
                    }
                }

                // Execute dragging
                if self.dragging {
                    if let Some(entity) = self.selected_entity {
                        if let Some(position) = world_ecs.get_store_mut::<Position>().get_mut(entity) {
                            let mouse_world = coord::mouse_world_pos(camera);
                            position.position = mouse_world + self.drag_offset;
                        }
                    }
                    if is_mouse_button_released(MouseButton::Left) {
                        // Drag finished
                        if let (Some(entity), Some(start_pos)) = (self.selected_entity, self.drag_start_position.take()) {
                            // Read the final position
                            if let Some(final_pos) = world_ecs
                                .get_store::<Position>()
                                .get(entity)
                                .map(|p| p.position)
                            {
                                // Only push the command if the entity changed its location
                                if (final_pos - start_pos).length_squared() > 0.0 {
                                    push_command(Box::new(
                                        MoveEntityCmd::new(entity, start_pos, final_pos),
                                    ));
                                }
                            }
                        }
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

                if Controls::v() && !input_is_focused() {
                    self.view_preview = !self.view_preview;
                }
            }
        }

        if is_key_pressed(KeyCode::Tab) && !input_is_focused() {
            self.mode = match self.mode {
                RoomEditorMode::Tilemap => RoomEditorMode::Scene,
                RoomEditorMode::Scene => RoomEditorMode::Tilemap,
            };
        }

        if is_key_pressed(KeyCode::G) && !input_is_focused() {
            self.show_grid = !self.show_grid;
        }

        if is_key_pressed(KeyCode::R) && !input_is_focused() {
            EditorCameraController::reset_editor_camera(camera, room);
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
                    room.position,
                );

                if self.show_grid { 
                    set_camera(camera);
                    grid::draw_grid(camera);
                }
            }
            RoomEditorMode::Scene => {
                let room_camera = Room::get_room_camera(world_ecs, room.id)
                    .expect("This room should have a camera.");

                let render_cam = if self.view_preview {
                    &room_camera.camera
                } else {
                    camera
                };

                self.inspector.set_rect(inspector_rect);

                // Draws everything in the room. Same implementation as the game.
                render_room(
                    world_ecs, 
                    room, 
                    asset_manager,
                    &mut self.light_system,
                    render_cam,
                );

                if !self.view_preview {
                    set_camera(camera);

                    if self.show_grid { 
                        grid::draw_grid(camera);
                    }
                    
                    draw_camera_placeholder(room_camera.position);
                    draw_light_placeholders(world_ecs);
                    draw_glow_placeholders(world_ecs, asset_manager);

                    if let Some(selected_entity) = self.selected_entity {
                        if !world_ecs.has_any::<(RoomCamera, Light)>(selected_entity) {
                            highlight_selected_entity(world_ecs, selected_entity, asset_manager, YELLOW);
                        }

                        draw_collider(world_ecs, selected_entity);
                        self.draw_camera_viewport(camera, world_ecs, selected_entity);
                    }

                    set_default_camera();
                
                    // If an entity is selected, forward it to the inspector.
                    if let Some(entity) = self.selected_entity {
                        self.inspector.set_target(Some(entity));
                    } else {
                        self.inspector.set_target(None); // clears the panel
                    }
                    
                    self.create_entity_requested = self.inspector.draw(
                        asset_manager, 
                        world_ecs,
                        room,
                    );

                    if self.inspector.target.is_none() {
                        self.selected_entity = None;
                    }
                }
            }
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

    pub fn is_mouse_over_ui(&self) -> bool {
        self.inspector.is_mouse_over()
    }

    pub fn set_selected_entity(&mut self, entity: Option<Entity>) {
        self.selected_entity = entity;
        self.inspector.set_target(entity);
    }

    pub fn reset(&mut self) {
        self.mode = RoomEditorMode::Scene;
        self.selected_entity = None;
        self.initialized = false;
        self.request_play = false;
        self.view_preview = false
    }
}

pub fn can_drag_entity(world_ecs: &WorldEcs, entity: Entity) -> bool {
    if world_ecs.get_store::<TileSprite>().get(entity).is_some() {
        return false;
    }
    true
}