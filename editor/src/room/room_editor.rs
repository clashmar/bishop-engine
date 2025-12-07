// editor/src/room/room_editor.rs
use engine_core::game::game::*;
use crate::gui::modal::is_modal_open;
use crate::gui::mode_selector::*;
use crate::editor_assets::editor_assets::*;
use crate::room::room_editor_rendering::*;
use crate::commands::entity_commands::*;
use crate::global::*;
use crate::gui::inspector::inspector_panel::InspectorPanel;
use crate::tilemap::tilemap_editor::TileMapEditor;
use crate::world::coord;
use crate::canvas::grid;
use crate::editor_camera_controller::EditorCameraController;
use engine_core::controls::controls::Controls;
use engine_core::world::world::World;
use macroquad::miniquad::CursorIcon;
use macroquad::miniquad::window::set_mouse_cursor;
use engine_core::ui::widgets::*;
use engine_core::animation::animation_system::*;
use engine_core::rendering::render_room::*;
use engine_core::world::room::*;
use engine_core::input::*;
use engine_core::global::*;
use engine_core::camera::game_camera::*;
use macroquad::prelude::*;
use engine_core::assets::asset_manager::AssetManager;
use engine_core::ecs::world_ecs::WorldEcs;
use engine_core::ecs::entity::Entity;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::ecs::component::*;
use engine_core::lighting::light::Light;
use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
            RoomEditorMode::Tilemap => &TILE_ICON,
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
    pub inspector: InspectorPanel,
    pub selected_entity: Option<Entity>,
    active_rects: Vec<Rect>,
    show_grid: bool,
    drag_offset: Vec2,
    dragging: bool,
    drag_start_position: Option<Vec2>,
    initialized: bool, 
    pub create_entity_requested: bool,
    pub request_play: bool,
    pub view_preview: bool,
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
            inspector: InspectorPanel::new(),
            selected_entity: None,
            active_rects: Vec::new(),
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

    pub async fn update(
        &mut self, 
        camera: &mut Camera2D,
        room_id: RoomId,
        current_world: &mut World,
        asset_manager: &mut AssetManager,
    ) {
        let other_bounds: Vec<(Vec2, Vec2)> = current_world.rooms
            .iter()
            .filter(|r| r.id != room_id)
            .map(|r| (r.position, r.size))
            .collect();

        let world_ecs = &mut current_world.world_ecs;
        
        let room = current_world.rooms
            .iter_mut()
            .find(|r| r.id == room_id)
            .expect("Could not find room in world.");

        if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
            clear_all_input_focus();
        }

        if !self.initialized {
            EditorCameraController::reset_room_editor_camera(camera, room);
            self.initialized = true;
        }

        self.handle_mouse_cursor();

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
                ).await;
            }
            RoomEditorMode::Scene => {
                if self.ui_was_clicked() {
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
                    let entity = &mut current_world.world_ecs
                        .create_entity()
                        .with(Position { position: room.position })
                        .with(CurrentRoom(room.id))
                        .finish();

                    // Immediately select it so the inspector shows the newly‑created entity
                    self.selected_entity = Some(*entity);
                    self.create_entity_requested = false;
                }

                // If an entity is selected, forward it to the inspector
                if let Some(entity) = self.selected_entity {
                    self.inspector.set_target(Some(entity));
                } else {
                    self.inspector.set_target(None); // Clears the panel
                }

                if self.inspector.target.is_none() {
                    self.selected_entity = None;
                }
            }
        }

        self.handle_shortcuts(camera, room);
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

        let mut game_ctx = game.ctx();
        let world_ecs = &mut game_ctx.cur_world_ecs;
        let room = &mut game_ctx.cur_room;
        let asset_manager = &mut game_ctx.asset_manager;

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
                self.tilemap_editor.tilemap_panel.set_rect(inspector_rect);
                self.tilemap_editor.draw(
                    camera, 
                    tilemap, 
                    exits, 
                    world_ecs,
                    asset_manager,
                    room.position,
                ).await;

                if self.show_grid { 
                    set_camera(camera);
                    grid::draw_grid(camera);
                }
            }
            RoomEditorMode::Scene => {
                // TODO: Pick best camera for preview from room cameras
                let room_camera = get_room_camera(world_ecs, room_id)
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
                    
                    draw_camera_placeholders(&world_ecs, room_id);
                    draw_light_placeholders(world_ecs, room_id);
                    draw_glow_placeholders(world_ecs, asset_manager, room_id);

                    if let Some(selected_entity) = self.selected_entity {
                        if !world_ecs.has_any::<(RoomCamera, Light)>(selected_entity) {
                            highlight_selected_entity(world_ecs, selected_entity, asset_manager, YELLOW);
                        }

                        draw_collider(world_ecs, selected_entity);
                        self.draw_camera_viewport(camera, world_ecs, selected_entity);
                    }
                }
            }
        }

        // Scene UI
        self.draw_coordinates(camera, room);
        self.draw_ui(&mut game_ctx);
    }

    /// Handles mouse selection / movement.
    fn handle_selection(
        &mut self,
        room_id: RoomId,
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
            return true; // Drag handled this frame
        }
        false // No active drag
    }

    /// Moves the currently selected entity by one pixel.
    fn handle_keyboard_move(
        &mut self,
        world_ecs: &mut WorldEcs,
        room_id: RoomId,
    ) {
        // Only act when an entity is selected and no drag is in progress
        if self.dragging 
        || self.selected_entity.is_none()
        || input_is_focused() {
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

    pub fn set_selected_entity(&mut self, entity: Option<Entity>) {
        self.selected_entity = entity;
        self.inspector.set_target(entity);
    }

    fn handle_shortcuts(&mut self, camera: &mut Camera2D, room: &mut Room) {
        // Shortcuts for both tilemap and scene
        if Controls::g() && !input_is_focused() {
            self.show_grid = !self.show_grid;
        }

        if Controls::r() && !input_is_focused() {
            EditorCameraController::reset_room_editor_camera(camera, room);
        }

        for mode in RoomEditorMode::iter() {
            if let Some(is_pressed) = mode.shortcut() {
                if is_pressed() && !input_is_focused() {
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
                if Controls::v() && !input_is_focused() {
                    self.view_preview = !self.view_preview;
                }

                if Controls::paste() {
                    push_command(Box::new(PasteEntityCmd::new()));
                }
            }
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
        || self.inspector.is_mouse_over() // Inspector has its own check
        || is_dropdown_open()
        || is_modal_open()
    }

    fn ui_was_clicked(&self) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && self.is_mouse_over_ui()
    }

    fn handle_mouse_cursor(&self) {
        if self.is_mouse_over_ui() {
            set_mouse_cursor(CursorIcon::Default);
        } else {
            match self.mode {
                RoomEditorMode::Scene => {
                    set_mouse_cursor(CursorIcon::Default);
                }
                RoomEditorMode::Tilemap => {
                    set_mouse_cursor(CursorIcon::Crosshair);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.tilemap_editor.reset();
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
    room_id: RoomId,
) -> bool {
    // Make sure the entity is in the requested room
    match world_ecs.get_store::<CurrentRoom>().get(entity) {
        Some(CurrentRoom(id)) => *id == room_id,
        None => false,
    }
}

/// A slice of all the modes.
static ALL_MODES: Lazy<&'static [RoomEditorMode]> = Lazy::new(|| {
    Box::leak(Box::new(
        RoomEditorMode::iter().collect::<Vec<_>>()
    ))
});