// editor/src/room/room_editor.rs
use engine_core::{
    animation::animation_system::update_animation_sytem, assets::asset_manager::AssetManager, camera::game_camera::{RoomCamera, get_room_camera}, ecs::{
        component::{CurrentRoom, Position}, 
        entity::Entity, 
        world_ecs::WorldEcs
    }, global::tile_size, input::get_omni_input_pressed, lighting::light::Light, rendering::{render_room::*, render_system::RenderSystem}, ui::widgets::*, world::room::Room
};
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
            clear_all_input_focus(); // TODO: Find a way to clear focus even when over ui
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

                let drag_handled = self.handle_selection(
                    room.id,
                    camera,
                    world_ecs,
                    asset_manager,
                    mouse_screen,
                    ui_was_clicked,
                );

                if !drag_handled {
                    self.handle_keyboard_move(world_ecs, room.id);
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
        asset_manager: &mut AssetManager,
        render_system: &mut RenderSystem,
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
                // TODO: Pick best camera for preview from room cameras
                let room_camera = get_room_camera(world_ecs, room.id)
                    .expect("This room should have at least one camera.");

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
                    render_system,
                    render_cam,
                    0.0,
                    None,
                );

                // Present room depending on view mode
                if self.view_preview {
                    render_system.present_game();
                } else {
                    set_default_camera();
                    render_system.draw_pass(
                        &render_system.final_comp_mat, 
                        &render_system.final_comp_rt.texture
                    );
                }

                if !self.view_preview {
                    set_camera(camera);

                    if self.show_grid { 
                        grid::draw_grid(camera);
                    }
                    
                    draw_camera_placeholders(&world_ecs, room.id);
                    draw_light_placeholders(world_ecs, room.id);
                    draw_glow_placeholders(world_ecs, asset_manager, room.id);

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

    /// Handles mouse selection / movement.
    fn handle_selection(
        &mut self,
        room_id: usize,
        camera: &Camera2D,
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
        mouse_screen: Vec2,
        ui_was_clicked: bool,
    ) -> bool {
        if !ui_was_clicked
            && is_mouse_button_pressed(MouseButton::Left)
            && !self.dragging
        {
            self.selected_entity = None;
            for (entity, pos) in world_ecs.get_store::<Position>().data.iter() {
                // Skip tiles, UI etc
                if !can_select_entity_in_room(world_ecs, *entity, room_id) {
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

        // Execute the drag while the button is held
        if self.dragging {
            if let Some(entity) = self.selected_entity {
                let (w, h) = entity_dimensions(world_ecs, asset_manager, entity);
                if let Some(position) = world_ecs
                    .get_store_mut::<Position>()
                    .get_mut(entity)
                {
                    let mouse_world = coord::mouse_world_pos(camera);
                    let mut new_pos = mouse_world + self.drag_offset;

                    // Snap to grid while S is held
                    if is_key_down(KeyCode::S) {
                        let tile = (mouse_world / tile_size()).floor();
                        let tile_center_x = tile.x * tile_size() + tile_size() * 0.5;
                        let tile_bottom_y = tile.y * tile_size() + tile_size();
                        new_pos = vec2(tile_center_x - w * 0.5, tile_bottom_y - h);
                    }
                    position.position = new_pos;
                }
            }

            // Finish the drag when the button is released
            if is_mouse_button_released(MouseButton::Left) {
                if let (Some(entity), Some(start_pos)) =
                    (self.selected_entity, self.drag_start_position.take())
                {
                    // Final position after the drag
                    if let Some(final_pos) = world_ecs
                        .get_store::<Position>()
                        .get(entity)
                        .map(|p| p.position)
                    {
                        // Push a command only if the entity actually moved
                        if (final_pos - start_pos).length_squared() > 0.0 {
                            push_command(Box::new(MoveEntityCmd::new(
                                entity, start_pos, final_pos,
                            )));
                        }
                    }
                }
                self.dragging = false;
            }
            return true; // drag handled this frame
        }
        false // no active drag
    }

    /// Moves the currently selected entity by one pixel.
    fn handle_keyboard_move(
        &mut self,
        world_ecs: &mut WorldEcs,
        room_id: usize,
    ) {
        // Only act when an entity is selected and no drag is in progress
        if self.dragging || self.selected_entity.is_none() {
            return;
        }

        let dir = get_omni_input_pressed();
        if dir.length_squared() == 0.0 {
            return;
        }

        // Move exactly one pixel
        let step = dir;
        let entity = self.selected_entity.unwrap();

        // Make sure the entity is moveable
        if !can_select_entity_in_room(world_ecs, entity, room_id) {
            return;
        }

        if let Some(position) = world_ecs.get_store_mut::<Position>().get_mut(entity) {
            let old = position.position;
            position.position += step;

            push_command(Box::new(MoveEntityCmd::new(
                entity,
                old,
                position.position,
            )));
        }
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

pub fn can_select_entity_in_room(
    world_ecs: &WorldEcs,
    entity: Entity,
    room_id: usize,
) -> bool {
    // Make sure the entity is in the requested room
    match world_ecs.get_store::<CurrentRoom>().get(entity) {
        Some(CurrentRoom(id)) => *id == room_id,
        None => false,
    }
}