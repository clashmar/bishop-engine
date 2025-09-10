// editor/src/room/room_editor.rs
use crate::{
    camera_controller::CameraController, 
    canvas::grid, 
    gui::inspector::panel::InspectorPanel, 
    tilemap::tilemap_editor::TileMapEditor, 
    world::coord
};
use engine_core::{
    assets::{asset_manager::AssetManager, sprite::Sprite}, 
    constants::*, 
    ecs::{component::Position, entity::Entity, world_ecs::WorldEcs}, 
    tiles::{tile::TileSprite, tilemap::TileMap}, 
    world::room::{Room, RoomMetadata}
};
use macroquad::prelude::*;
use uuid::Uuid;

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
        }
    }

    /// Returns `true` if user wants to exit back to world view.  
    pub fn update(
        &mut self, 
        camera: &mut Camera2D,
        room: &mut Room,
        room_id: Uuid, 
        rooms_metadata: &mut [RoomMetadata],
        world_ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
        world_id: &Uuid,
    ) -> bool {
        let tilemap = &mut room.variants[0].tilemap;

        if !self.initialized {
            CameraController::reset_room_camera(camera, tilemap);
            self.initialized = true;
        }

        let inspector_rect = Rect::new(
            screen_width() * 0.75, 
            0.0,                  
            screen_width() * 0.25, 
            screen_height(),       
        );

        // Click‑selection
        let mouse_screen: Vec2 = mouse_position().into();

        let mut ui_was_clicked = false;
        
        if inspector_rect.contains(mouse_screen) && is_mouse_button_pressed(MouseButton::Left) {
            ui_was_clicked = true;
        }

        match self.mode {
            RoomEditorMode::Tilemap => {
                // Collect bounds for all other rooms to check for intersections
                let other_bounds: Vec<(Vec2, Vec2)> = rooms_metadata
                    .iter()
                    .filter(|m| m.id != room_id)
                    .map(|m| (m.position, m.size))
                    .collect();

                let room_metadata = rooms_metadata
                    .iter_mut()
                    .find(|m| m.id == room_id)
                    .expect("metadata must still exist");

                self.tilemap_editor.update(
                    camera, 
                    tilemap, 
                    room_metadata, 
                    &other_bounds, 
                    world_ecs, 
                    asset_manager
                );
            }
            RoomEditorMode::Scene => {
                let room_metadata = rooms_metadata
                    .iter_mut()
                    .find(|m| m.id == room_id)
                    .expect("metadata must still exist");

                if !ui_was_clicked && is_mouse_button_pressed(MouseButton::Left) && !self.dragging {
                    self.selected_entity = None;
                    for (entity, position) in world_ecs.get_store::<Position>().data.iter() {
                        let room_position = position.position - room_metadata.position;
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
                            position.position = mouse_world + room_metadata.position + self.drag_offset;
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
                        .with(Position { position: room_metadata.position })
                        .finish();

                    // Immediately select it so the inspector shows the newly‑created entity
                    self.selected_entity = Some(entity);
                    self.inspector.set_target(Some(entity));
                    self.create_entity_requested = false;
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.tilemap_editor.reset();
            self.reset();
            return true;
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
            CameraController::reset_room_camera(camera, tilemap);
        }

        false
    }

    pub fn draw(
        &mut self, 
        camera: &Camera2D,
        room: &Room,
        room_metadata: &RoomMetadata,
        world_ecs: &mut WorldEcs, 
        asset_manager: &mut AssetManager
    ) {
        let tilemap = &room.variants[0].tilemap;
        let exits = &room_metadata.exits;

        match self.mode {
            RoomEditorMode::Tilemap => {
                self.tilemap_editor.draw(
                    camera, 
                    tilemap, 
                    exits, 
                    world_ecs,
                    asset_manager,
                );
            }
            RoomEditorMode::Scene => {
                tilemap.draw(camera, exits, world_ecs, asset_manager);
                self.draw_entities(world_ecs, room_metadata, tilemap, asset_manager);
                set_default_camera();
                
                // Inspector
                let inspector_rect = Rect::new(
                    screen_width() * 0.75, 
                    0.0, 
                    screen_width() * 0.25, 
                    screen_height()
                );

                self.inspector.set_rect(inspector_rect);

                // If an entity is selected, forward it to the inspector.
                if let Some(entity) = self.selected_entity {
                    self.inspector.set_target(Some(entity));
                } else {
                    self.inspector.set_target(None); // clears the panel
                }
                
                self.create_entity_requested = self.inspector.draw(asset_manager, world_ecs);
            }
        }

        if self.show_grid {
            set_camera(camera);
            grid::draw_grid(camera);
        }

        set_default_camera();
        self.draw_coordinates(camera, room_metadata);
    }

    fn draw_entities(
        &self, 
        world_ecs: &WorldEcs, 
        room_metadata: &RoomMetadata, 
        tilemap: &TileMap,
        asset_manager: &mut AssetManager,
    ) {
        let room_min = room_metadata.position;
        let room_max = room_min
            + vec2(
                tilemap.width  as f32 * TILE_SIZE,
                tilemap.height as f32 * TILE_SIZE,
            );

        for (entity, pos) in world_ecs.get_store::<Position>().data.iter() {
            // Skip tiles
            if world_ecs.get_store::<TileSprite>().get(*entity).is_some() {
                continue;
            }

            // Only draw the placeholder if the entity lies inside the current
            // room’s bounds (after the offset has been applied).
            if pos.position.x >= room_min.x
                && pos.position.x <= room_max.x
                && pos.position.y >= room_min.y 
                && pos.position.y <= room_max.y
            {
                // Adjust the drawing position of the entity
                let room_pos = pos.position - room_metadata.position;
                
                // Draw the sprite (if the entity has a Sprite component)
                if let Some(sprite) = world_ecs.get_store::<Sprite>().get(*entity) {
                    let tex = asset_manager.get_texture_from_id(sprite.sprite_id);
                    // Draw the texture centred on the entity’s position.
                    draw_texture_ex(
                        tex,
                        room_pos.x - TILE_SIZE / 2.0,
                        room_pos.y - TILE_SIZE / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else {
                    // Fallback placeholder (magenta square)
                    draw_rectangle(
                        room_pos.x - 10.0,
                        room_pos.y - 10.0,
                        20.0,
                        20.0,
                        MAGENTA,
                    );
                }
            }
        }
        
        // Highlight the currently selected entity (yellow box)
        if let Some(sel) = self.selected_entity {
            if let Some(pos) = world_ecs.get_store::<Position>().get(sel) {
                draw_rectangle_lines(
                    pos.position.x - room_metadata.position.x - 11.0,
                    pos.position.y - room_metadata.position.y - 11.0,
                    22.0,
                    22.0,
                    2.0,
                    YELLOW,
                );
            }
        }
    }

    pub fn reset(&mut self) {
        self.mode = RoomEditorMode::Scene;
        self.selected_entity = None;
        self.initialized = false;
    }
}