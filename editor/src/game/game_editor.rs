// editor/src/game/game_editor.rs
use engine_core::ui::widgets::*;
use std::collections::HashMap;
use macroquad::prelude::*;
use engine_core::world::world::WorldId;
use engine_core::game::game::Game;

const WORLD_RADIUS: f32 = 60.0;

pub enum GameEditorMode {
    Default,
    Edit,
}
pub struct GameEditor {
    mode: GameEditorMode,
    dragged_world: Option<WorldId>,
    drag_offset: Vec2,
    world_widget_ids: HashMap<WorldId, WidgetId>,
}

impl GameEditor {
    pub fn new() -> Self {
        Self { 
            mode: GameEditorMode::Default,
            dragged_world: None,
            drag_offset: Vec2::ZERO,
            world_widget_ids: HashMap::new(),
        }
    }

    pub async fn update(&mut self, game: &mut Game) -> Option<WorldId> {
        

        match self.mode {
            GameEditorMode::Default => {
                // Select world
                if is_mouse_button_pressed(MouseButton::Left) {
                    let mouse: Vec2 = mouse_position().into();

                    for world in &game.worlds {
                        if (mouse - world.map_position).length() <= WORLD_RADIUS {
                            return Some(world.id);
                        }
                    }
                }
            },
            GameEditorMode::Edit => {
                // Drag world
                self.handle_drag_start(game);
                self.handle_drag_move(game);
            }
        }

        None
    }

    pub fn draw(&mut self, game: &mut Game) {
        clear_background(BLACK);
        self.draw_worlds(game);
    }

    fn draw_worlds(&mut self, game: &mut Game) {
        for world in &game.worlds {
            // Circle
            draw_circle(
                world.map_position.x,
                world.map_position.y,
                WORLD_RADIUS,
                WHITE,
            );
        }

        // Name widget
        for world in &mut game.worlds {
            let rect = Rect::new(
                world.map_position.x - WORLD_RADIUS,
                world.map_position.y - 12.0,
                WORLD_RADIUS * 2.0,
                24.0,
            );

            let widget_id = self.widget_id_for_world(world.id);

            let (new_name, _focused) = gui_input_text_default(
                widget_id,
                rect,
                &world.name,
            );

            if new_name != world.name {
                world.name = new_name;
            }
        }
    }

    fn handle_drag_start(&mut self, game: &Game) {
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse: Vec2 = mouse_position().into();

            for world in &game.worlds {
                if (mouse - world.map_position).length() <= WORLD_RADIUS {
                    self.dragged_world = Some(world.id);
                    self.drag_offset = world.map_position - mouse;
                    break;
                }
            }
        }
    }

    fn handle_drag_move(&mut self, game: &mut Game) {
        if let Some(id) = self.dragged_world {
            if is_mouse_button_down(MouseButton::Left) {
                let mouse: Vec2 = mouse_position().into();
                
                if let Some(world) = game.worlds.iter_mut().find(|w| w.id == id) {
                    world.map_position = mouse + self.drag_offset;
                }
            } else {
                self.dragged_world = None;
            }
        }
    }

    fn widget_id_for_world(&mut self, world_id: WorldId) -> WidgetId {
        *self.world_widget_ids.entry(world_id).or_insert_with(WidgetId::default)
    }
}