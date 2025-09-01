use crate::{gui::add_entity_button::AddEntityButton, tilemap::tilemap_editor::TileMapEditor, world::coord};
use core::{assets::asset_manager::AssetManager, constants::*, ecs::{entity::Entity, world_ecs::WorldEcs}, world::room::{Room, RoomMetadata}};
use macroquad::prelude::*;
use uuid::Uuid;

pub enum RoomEditorMode {
    Tilemap,
    Scene,
}

pub struct RoomEditor {
    pub mode: RoomEditorMode,
    pub tilemap_editor: TileMapEditor,
    add_entity_btn: AddEntityButton,
    selected_entity: Option<Entity>,
    drag_offset: Vec2,
    dragging: bool,
}

impl RoomEditor {
    pub fn new() -> Self {
        Self {
            mode: RoomEditorMode::Tilemap,
            tilemap_editor: TileMapEditor::new(),
            add_entity_btn: AddEntityButton::new(),
            selected_entity: None,
            drag_offset: Vec2::ZERO,
            dragging: false,
        }
    }

    /// Returns `true` if user wants to exit back to world view.  
    pub fn update(
        &mut self, 
        camera: &mut Camera2D,
        room: &mut Room,
        room_id: Uuid, 
        rooms_metadata: &mut [RoomMetadata],
        ecs: &mut WorldEcs,
        asset_manager: &mut AssetManager,
    ) -> bool {
        futures::executor::block_on(
            self.tilemap_editor.palette.process_create_request(ecs, asset_manager)
        );
        match self.mode {
            RoomEditorMode::Tilemap => {
                // Collect bounds for all other rooms to check for intersections
                let other_bounds: Vec<(Vec2, Vec2)> = rooms_metadata
                    .iter()
                    .filter(|m| m.id != room_id)
                    .map(|m| (m.position, m.size))
                    .collect();

                let tilemap = &mut room.variants[0].tilemap;

                let room_metadata = rooms_metadata
                    .iter_mut()
                    .find(|m| m.id == room_id)
                    .expect("metadata must still exist");

                self.tilemap_editor.update(camera, tilemap, room_metadata, &other_bounds, ecs);
            }
            RoomEditorMode::Scene => {
                // Click‑selection
                let mouse_screen: Vec2 = mouse_position().into();

                if is_mouse_button_pressed(MouseButton::Left) && !self.dragging {
                    self.selected_entity = None;
                    for (entity, position) in ecs.positions.data.iter() {
                        let screen = coord::world_to_screen(camera, position.position);
                        let hit = Rect::new(screen.x - 10.0, screen.y - 10.0, 20.0, 20.0);
                        if hit.contains(mouse_screen) {
                            self.selected_entity = Some(*entity);
                            let mouse_world = coord::mouse_world_pos(camera);
                            self.drag_offset = position.position - mouse_world;
                            self.dragging = true;
                            break;
                        }
                    }
                }

                // Dragging
                if self.dragging {
                    if let Some(entity) = self.selected_entity {
                        if let Some(position) = ecs.positions.get_mut(entity) {
                            let mouse_world = coord::mouse_world_pos(camera);
                            position.position = mouse_world + self.drag_offset;
                        }
                    }
                    if is_mouse_button_released(MouseButton::Left) {
                        self.dragging = false;
                    }
                }

                let room_metadata = rooms_metadata
                    .iter_mut()
                    .find(|m| m.id == room_id)
                    .expect("metadata must still exist");

                self.add_entity_btn.try_click(ecs, room_metadata);
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

        false
    }

    pub fn draw(
        &mut self, 
        camera: &Camera2D,
        room: &Room,
        room_metadata: &RoomMetadata,
        ecs: &WorldEcs, 
        asset_manager: &mut AssetManager
    ) {
        match self.mode {
            RoomEditorMode::Tilemap => {
                let tilemap = &room.variants[0].tilemap;
                let exits = &room_metadata.exits;
                self.tilemap_editor.draw(
                    camera, 
                    tilemap, 
                    exits, 
                    ecs,
                    asset_manager,
                );
            }
            RoomEditorMode::Scene => {
                let tilemap = &room.variants[0].tilemap;
                let exits = &room_metadata.exits;
                tilemap.draw(camera, exits, ecs, asset_manager);

                let room_min = room_metadata.position;
                let room_max = room_min
                    + vec2(
                        tilemap.width  as f32 * TILE_SIZE,
                        tilemap.height as f32 * TILE_SIZE,
                    );
                for (_e, pos) in ecs.positions.data.iter() {
                    if pos.position.x >= room_min.x
                        && pos.position.x <= room_max.x
                        && pos.position.y >= room_min.y
                        && pos.position.y <= room_max.y
                    {
                        draw_entity_placeholder(pos.position);
                    }
                }
                // Draw highlight
                if let Some(sel) = self.selected_entity {
                    if let Some(p) = ecs.positions.get(sel) {
                        draw_rectangle_lines(
                            p.position.x - 11.0,
                            p.position.y - 11.0,
                            22.0,
                            22.0,
                            2.0,
                            YELLOW,
                        );
                    }
                }
                set_default_camera();
                self.add_entity_btn.draw();
            }
        }
        set_default_camera();
        self.draw_coordinates(camera, room_metadata);
    }

    pub fn reset(&mut self) {
        self.mode = RoomEditorMode::Tilemap;
        self.selected_entity = None;
    }
}

fn draw_entity_placeholder(pos: Vec2) {
    draw_rectangle(
        pos.x - 10.0,
        pos.y - 10.0,
        20.0,
        20.0,
        MAGENTA,
    );
}