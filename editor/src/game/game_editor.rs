// editor/src/game/game_editor.rs
use crate::gui::gui_constants::*;
use crate::miniquad::CursorIcon;
use macroquad::miniquad::window::set_mouse_cursor;
use once_cell::sync::Lazy;
use crate::controls::controls::Controls;
use crate::gui::menu_panel::*;
use crate::gui::mode_selector::ModeSelector;
use crate::editor_assets::editor_assets::*;
use crate::gui::mode_selector::ModeInfo;
use engine_core::ui::widgets::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::collections::HashMap;
use macroquad::prelude::*;
use engine_core::world::world::WorldId;
use engine_core::game::game::Game;

const WORLD_RADIUS: f32 = 60.0;

#[derive(Copy, Clone, PartialEq, EnumIter)]
pub enum GameEditorMode {
    Select,
    Edit,
}

impl ModeInfo for GameEditorMode {
    fn label(&self) -> &'static str {
        match self {
            GameEditorMode::Select => "Select: S",
            GameEditorMode::Edit => "Edit: E",
        }
    }
    fn icon(&self) -> &'static Texture2D {
        match self {
            GameEditorMode::Select => &SELECT_ICON,
            GameEditorMode::Edit => &EDIT_ICON,
        }
    }
    fn shortcut(self) -> Option<fn() -> bool> {
        match self {
            GameEditorMode::Select => Some(Controls::s),
            GameEditorMode::Edit => Some(Controls::e),
        }
    }
}

pub struct GameEditor {
    mode: GameEditorMode,
    mode_selector: ModeSelector<GameEditorMode>,
    active_rects: Vec<Rect>,
    dragged_world: Option<WorldId>,
    drag_offset: Vec2,
    world_widget_ids: HashMap<WorldId, WidgetId>,
}

impl GameEditor {
    pub fn new() -> Self {
        let mode = GameEditorMode::Select;
        Self { 
            mode,
            mode_selector: ModeSelector {
                current: mode,
                options: *ALL_MODES,
            },
            active_rects: Vec::new(),
            dragged_world: None,
            drag_offset: Vec2::ZERO,
            world_widget_ids: HashMap::new(),
        }
    }

    pub async fn update(&mut self, game: &mut Game) -> Option<WorldId> {
        self.handle_mouse_cursor();

        match self.mode {
            GameEditorMode::Select => {
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

        self.handle_shortcuts();

        None
    }

    pub fn draw(&mut self, game: &mut Game) {
        self.active_rects.clear();
        clear_background(BLACK);
        self.draw_worlds(game);
        self.draw_ui();
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

        // Display name
        for world in &mut game.worlds {
            const NAME_HEIGHT: f32 = 24.0;
            let (x, width) = center_text(world.map_position.x, &world.name);

            let name_rect = Rect::new(
                x,
                world.map_position.y - WORLD_RADIUS - SPACING - NAME_HEIGHT,
                width,
                NAME_HEIGHT,
            );

            if self.mode == GameEditorMode::Edit {
                // Show text input widget
                let widget_id = self.widget_id_for_world(world.id);

                let (new_name, _focused) = gui_input_text_default(
                    widget_id,
                    name_rect,
                    &world.name,
                );

                if new_name != world.name {
                    world.name = new_name;
                }
            } else {
                // Just display the name
                draw_input_field_text(&world.name, name_rect);
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

    fn draw_ui(&mut self) {
        self.register_rect(draw_top_panel_full());

        if self.mode_selector.draw().1 {
            self.mode = self.mode_selector.current;
        }
    }

    fn handle_shortcuts(&mut self) {
        for mode in GameEditorMode::iter() {
            if let Some(is_pressed) = mode.shortcut() {
                if is_pressed() && !input_is_focused() {
                    self.mode = mode;
                    self.mode_selector.current = mode;
                    break;
                }
            }
        }
    }

    fn widget_id_for_world(&mut self, world_id: WorldId) -> WidgetId {
        *self.world_widget_ids.entry(world_id).or_insert_with(WidgetId::default)
    }

    #[inline]
    fn register_rect(&mut self, rect: Rect) -> Rect {
        self.active_rects.push(rect);
        rect
    }

    fn is_mouse_over_ui(&self) -> bool {
        let mouse_screen: Vec2 = mouse_position().into();
        self.active_rects.iter().any(|r| r.contains(mouse_screen))
    }

    fn handle_mouse_cursor(&self) {
        if self.is_mouse_over_ui() {
            set_mouse_cursor(CursorIcon::Default);
        } else {
            match self.mode {
                GameEditorMode::Select => {
                    set_mouse_cursor(CursorIcon::Pointer);
                }
                GameEditorMode::Edit => {
                    set_mouse_cursor(CursorIcon::Crosshair);
                }
            }
        }
    }
}

/// A slice of all the modes.
static ALL_MODES: Lazy<&'static [GameEditorMode]> = Lazy::new(|| {
    Box::leak(Box::new(
        GameEditorMode::iter().collect::<Vec<_>>()
    ))
});