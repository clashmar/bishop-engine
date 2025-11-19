// editor/src/game/game_editor.rs
use std::cell::RefCell;
use engine_core::assets::sprite::SpriteId;
use engine_core::assets::asset_manager::AssetManager;
use engine_core::ui::colors::HIGHLIGHT_GREEN;
use engine_core::world::world::World;
use crate::gui::gui_constants::*;
use crate::gui::inspector::modal::*;
use crate::miniquad::CursorIcon;
use macroquad::miniquad::window::set_mouse_cursor;
use once_cell::sync::Lazy;
use engine_core::controls::controls::Controls;
use crate::gui::menu_bar::*;
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

#[derive(Copy, Clone, PartialEq, EnumIter)]
pub enum GameEditorMode {
    Select,
    Edit,
    Move,
}

impl ModeInfo for GameEditorMode {
    fn label(&self) -> &'static str {
        match self {
            GameEditorMode::Select => "Select: S",
            GameEditorMode::Edit => "Edit: E",
            GameEditorMode::Move => "Move: M",
        }
    }
    fn icon(&self) -> &'static Texture2D {
        match self {
            GameEditorMode::Select => &SELECT_ICON,
            GameEditorMode::Edit => &EDIT_ICON,
            GameEditorMode::Move => &MOVE_ICON,
        }
    }
    fn shortcut(self) -> Option<fn() -> bool> {
        match self {
            GameEditorMode::Select => Some(Controls::s),
            GameEditorMode::Edit => Some(Controls::e),
            GameEditorMode::Move => Some(Controls::m),
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
    modal: Modal,
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
            modal: Modal::new(340.0, 140.0),
        }
    }

    pub async fn update(&mut self, game: &mut Game) -> Option<WorldId> {
        self.handle_mouse_cursor();

        match self.mode {
            GameEditorMode::Select => {
                // Select world
                if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
                    for world in &game.worlds {
                        let texture = self.resolve_world_texture(world, &mut game.asset_manager);
                        if self.is_mouse_over_world(world, texture) {
                            return Some(world.id);
                        }
                    }
                }
            },
            GameEditorMode::Edit => {
                // Edit modal, handles its own UI and closing
                if self.modal.is_open() {
                    // Do nothing
                } else if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
                    for world in &mut game.worlds {
                        let tex = self.resolve_world_texture(world, &mut game.asset_manager);
                        if self.is_mouse_over_world(world, tex) {
                            // Capture the world id and its current sprite id
                            let world_id = world.id;
                            let current_sprite = world.meta.sprite_id.unwrap_or(SpriteId(0));

                            // Open the modal
                            let modal_rect = self.modal.rect;

                            let widgets: Vec<BoxedWidget> = vec![ 
                                Box::new(move |asset_manager| {
                                    let picker_rect = Rect::new(
                                        modal_rect.x + 20.0,
                                        modal_rect.y + 50.0,
                                        modal_rect.w - 40.0,
                                        40.0,
                                    );
                                    let mut chosen = current_sprite;
                                    if gui_sprite_picker(picker_rect, &mut chosen, asset_manager) {
                                        // Store the result for the main draw loop
                                        WORLD_SPRITE_RESULT.with(|c| *c.borrow_mut() = Some((world_id, chosen)));
                                    }
                                })
                            ];
                            
                            self.modal.open(widgets);
                            break;
                        }
                    }
                }
            }
            GameEditorMode::Move => {
                if !self.is_mouse_over_ui() {
                    // Drag world
                    self.handle_drag_start(game);
                    self.handle_drag_move(game);
                }
            }
        }

        self.handle_shortcuts();

        None
    }

    pub fn draw(
        &mut self, 
        game: &mut Game
    ) {
        clear_background(BLACK);

        if self.modal.is_open() {
            self.active_rects.push(self.modal.rect)
        }

        self.draw_worlds(game);
        self.draw_ui(game);
    }

    fn draw_worlds(
        &mut self, 
        game: &mut Game,
    ) {
        // Draw world
        for world in &game.worlds {
            let texture = self.resolve_world_texture(world, &mut game.asset_manager);

            // Hover tint
            let tint = if self.is_mouse_over_world(world, texture) && !self.is_mouse_over_ui() {
                HIGHLIGHT_GREEN
            } else {
                WHITE
            }; 

            // Default is a circle
            draw_texture(
                texture,
                world.meta.position.x,
                world.meta.position.y,
                tint,
            );
        }

        // Display name
        for world in &mut game.worlds {
            const NAME_HEIGHT: f32 = 24.0;
            let center = world.meta.position.x + (CIRCLE_120PX.width() / 2.);
            let (x, width) = center_text_field(center, &world.name);

            let name_rect = Rect::new(
                x,
                world.meta.position.y - SPACING - NAME_HEIGHT,
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

    fn handle_drag_start(&mut self, game: &mut Game) {
        if is_mouse_button_pressed(MouseButton::Left) {
            for world in &game.worlds {
                let texture = self.resolve_world_texture(world, &mut game.asset_manager);
                if self.is_mouse_over_world(world, texture) {
                    self.dragged_world = Some(world.id);
                    let mouse: Vec2 = mouse_position().into();
                    self.drag_offset = world.meta.position - mouse;
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
                    world.meta.position = mouse + self.drag_offset;
                }
            } else {
                self.dragged_world = None;
            }
        }
    }

    fn draw_ui(&mut self, game: &mut Game) {
        self.active_rects.clear();
        self.register_rect(draw_top_panel_full());

        if self.mode_selector.draw().1 {
            self.mode = self.mode_selector.current;
        }

        // Draw modal last
        if self.modal.is_open() {
            // Pass the asset manager so any widget that needs assets can use it
            let clicked_outside = self.modal.draw(&mut game.asset_manager);
            if clicked_outside {
                self.modal.close();
            }

            // Handle results
            WORLD_SPRITE_RESULT.with(|c| {
                if let Some((world_id, new_sprite)) = c.borrow_mut().take() {
                    if let Some(world) = game.worlds.iter_mut().find(|w| w.id == world_id) {
                        world.meta.sprite_id = if new_sprite.0 == 0 {
                            None
                        } else {
                            Some(new_sprite)
                        };
                    }
                    self.modal.close();
                }
            });
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
        || is_dropdown_open()
        || is_modal_open()
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
                GameEditorMode::Move => {
                    set_mouse_cursor(CursorIcon::Move);
                }
            }
        }
    }

    fn is_mouse_over_world(&self, world: &World, world_texture: &Texture2D) -> bool {
        let mouse: Vec2 = mouse_position().into();
        self.world_texture_bounds(world, world_texture).contains(mouse)
    }

    fn world_texture_bounds(&self, world: &World, world_texture: &Texture2D) -> Rect {
        Rect::new(
            world.meta.position.x,
            world.meta.position.y,
            world_texture.width(),
            world_texture.height(),
        )
    }

    fn resolve_world_texture<'a>(
        &self, world: &World, 
        asset_manager: &'a mut AssetManager
    ) -> &'a Texture2D {
        let texture = if let Some(id) = world.meta.sprite_id {
            asset_manager.get_texture_from_id(id)
        } else {
            &CIRCLE_120PX
        };

        texture
    }
}

/// A slice of all the modes.
static ALL_MODES: Lazy<&'static [GameEditorMode]> = Lazy::new(|| {
    Box::leak(Box::new(
        GameEditorMode::iter().collect::<Vec<_>>()
    ))
});

thread_local! {
    static WORLD_SPRITE_RESULT: RefCell<Option<(WorldId, SpriteId)>> = RefCell::new(None);
}