// editor/src/game/game_editor.rs
use std::cell::RefCell;
use engine_core::assets::sprite::SpriteId;
use engine_core::assets::asset_manager::AssetManager;
use engine_core::ui::colors::HIGHLIGHT_GREEN;
use engine_core::ui::prompt::*;
use engine_core::ui::text::*;
use engine_core::world::world::World;
use crate::editor_camera_controller::EditorCameraController;
use crate::gui::gui_constants::*;
use crate::gui::inspector::modal::*;
use crate::miniquad::CursorIcon;
use crate::storage::editor_storage;
use crate::storage::editor_storage::create_new_world;
use crate::world::coord;
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
    Delete,
}

impl ModeInfo for GameEditorMode {
    fn label(&self) -> &'static str {
        match self {
            GameEditorMode::Select => "Select: S",
            GameEditorMode::Edit => "Edit: E",
            GameEditorMode::Move => "Move: M",
            GameEditorMode::Delete => "Delete: D",
        }
    }
    fn icon(&self) -> &'static Texture2D {
        match self {
            GameEditorMode::Select => &SELECT_ICON,
            GameEditorMode::Edit => &EDIT_ICON,
            GameEditorMode::Move => &MOVE_ICON,
            GameEditorMode::Delete => &DELETE_ICON,
        }
    }
    fn shortcut(self) -> Option<fn() -> bool> {
        match self {
            GameEditorMode::Select => Some(Controls::s),
            GameEditorMode::Edit => Some(Controls::e),
            GameEditorMode::Move => Some(Controls::m),
            GameEditorMode::Delete => Some(Controls::d),
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
    selected_world_id: Option<WorldId>,
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
            selected_world_id: None,
            modal: Modal::new(340.0, 140.0),
        }
    }

    pub async fn update(
        &mut self, 
        camera: &Camera2D,
        game: &mut Game
    ) -> Option<WorldId> {
        self.handle_mouse_cursor();

        match self.mode {
            GameEditorMode::Select => {
                // Select world
                if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
                    for world in &game.worlds {
                        let texture = self.resolve_world_texture(world, &mut game.asset_manager);
                        if self.is_mouse_over_world(camera, world, texture) {
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
                        let texture = self.resolve_world_texture(world, &mut game.asset_manager);
                        if self.is_mouse_over_world(camera, world, texture) {
                            // Capture the world data
                            let world_id = world.id;
                            let current_name = world.name.clone();  
                            let current_sprite = world.meta.sprite_id.unwrap_or(SpriteId(0));
                            let widget_id = self.widget_id_for_world(world_id);

                            self.modal = Modal::new(400.0, 300.0);
                            
                            let mut prompt = WorldEditPrompt::new(
                                world_id,
                                self.modal.rect, 
                                widget_id,
                                current_name,
                                current_sprite
                            );

                            let widgets: Vec<BoxedWidget> = vec![
                                Box::new(move |asset_manager| {
                                    if let Some(result) = prompt.draw(asset_manager) {
                                        EDIT_WORLD_RESULT.with(|c| *c.borrow_mut() = Some(result));
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
                    self.handle_drag_start(camera, game);
                    self.handle_drag_move(camera, game);
                }
            },
            GameEditorMode::Delete => {
                // Delete world
                if is_mouse_button_pressed(MouseButton::Left) && !self.is_mouse_over_ui() {
                    for world in &game.worlds {
                        let texture = self.resolve_world_texture(world, &mut game.asset_manager);
                        if self.is_mouse_over_world(camera, world, texture) {
                            self.selected_world_id = Some(world.id);
                            self.modal = Modal::open_confirm_modal(&DELETE_WORLD_RESULT);
                        }
                    }
                }
            }
        }

        self.handle_shortcuts();

        None
    }

    pub fn draw(
        &mut self, 
        camera: &Camera2D, 
        game: &mut Game,

    ) {
        set_camera(camera);
        clear_background(BLACK);

        if self.modal.is_open() {
            self.active_rects.push(self.modal.rect)
        }

        self.draw_worlds(camera, game);
        self.draw_ui(game);
    }

    fn draw_worlds(
        &mut self, 
        camera: &Camera2D,
        game: &mut Game,
    ) {
        // Draw world
        for world in &game.worlds {
            let texture = self.resolve_world_texture(world, &mut game.asset_manager);

            // Hover tint
            let tint = if self.is_mouse_over_world(camera, world, texture) 
            && !self.is_mouse_over_ui() && self.dragged_world.is_none() {
                match self.mode {
                    GameEditorMode::Delete => {
                        RED
                    }
                    _ => {
                        HIGHLIGHT_GREEN
                    }
                }
                
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

            draw_input_field_text(&world.name, name_rect);
        }
    }

    fn handle_drag_start(&mut self, camera: &Camera2D, game: &mut Game) {
        if is_mouse_button_pressed(MouseButton::Left) {
            for world in &game.worlds {
                let texture = self.resolve_world_texture(world, &mut game.asset_manager);
                if self.is_mouse_over_world(camera, world, texture) {
                    self.dragged_world = Some(world.id);
                    let world_mouse = coord::mouse_world_pos(camera);
                    self.drag_offset = world.meta.position - world_mouse;
                    break;
                }
            }
        }
    }

    fn handle_drag_move(&mut self, camera: &Camera2D, game: &mut Game) {
        if let Some(id) = self.dragged_world {
            if is_mouse_button_down(MouseButton::Left) {
                let world_mouse = coord::mouse_world_pos(camera);
                
                if let Some(world) = game.worlds.iter_mut().find(|w| w.id == id) {
                    world.meta.position = world_mouse + self.drag_offset;
                }
            } else {
                self.dragged_world = None;
            }
        }
    }

    fn draw_ui(&mut self, game: &mut Game) {
        set_default_camera();

        self.active_rects.clear();
        self.register_rect(draw_top_panel_full());

        if self.mode_selector.draw().1 {
            self.mode = self.mode_selector.current;
        }

        self.draw_menu_buttons(game);

        // Draw modal last
        if self.modal.is_open() {
            // Pass the asset manager so any widget that needs assets can use it
            let clicked_outside = self.modal.draw(&mut game.asset_manager);
            if clicked_outside {
                self.modal.close();
            }

            // Handle results
            EDIT_WORLD_RESULT.with(|c| {
                if let Some(result) = c.borrow_mut().take() {
                    // Apply any name change
                    if let Some(new_name) = result.name {
                        if let Some(world) = game.worlds.iter_mut().find(|w| w.id == result.id) {
                            world.name = new_name;
                        }
                    }
                    // Apply any sprite change
                    if let Some(new_sprite) = result.sprite {
                        if let Some(world) = game.worlds.iter_mut().find(|w| w.id == result.id) {
                            world.meta.sprite_id = if new_sprite.0 == 0 {
                                None
                            } else {
                                Some(new_sprite)
                            };
                        }
                    }
                    self.modal.close();
                }
            });

            DELETE_WORLD_RESULT.with(|c| {
                if let Some(result) = c.borrow_mut().take() {
                    match result {
                        ConfirmPromptResult::Confirmed => {
                            if let Some(id) = self.selected_world_id {
                                game.delete_world(id);
                                let _ = editor_storage::save_game(game);
                            }
                        },
                        ConfirmPromptResult::Cancelled => { }
                    }
                    self.selected_world_id = None;
                    self.modal.close();
                }
            });
        }
    }

    fn draw_menu_buttons(&mut self, game: &mut Game) {
        const BTN_MARGIN: f32 = 10.0;

        let create_label = "New World";
        let txt_create = measure_text_ui(create_label, HEADER_FONT_SIZE_20, 1.0);
        let create_btn = Rect::new(
            screen_width() - txt_create.width - BTN_MARGIN - PADDING,
            BTN_MARGIN,
            txt_create.width + PADDING,
            BTN_HEIGHT,
        );

        if menu_button(create_btn, create_label, false) {
            game.add_world(create_new_world());
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
                GameEditorMode::Delete => {
                    set_mouse_cursor(CursorIcon::Crosshair);
                }
            }
        }
    }

    fn is_mouse_over_world(
        &self,
        camera: &Camera2D,
        world: &World, 
        world_texture: &Texture2D
    ) -> bool {
        let world_mouse = coord::mouse_world_pos(camera);
        self.world_texture_bounds(world, world_texture).contains(world_mouse)
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

    /// Sets the default camera for the game editor.
    pub fn init_camera(&self, camera: &mut Camera2D, game: &mut Game) {
        let (min, max) = self.world_bounds(game);
        let center = (min + max) * 0.5;
        let size = max - min;

        // Get the zoom for the whole area 
        let zoom = EditorCameraController::zoom_for_size(size, 2.0);

        // Apply the results
        camera.target = center;
        camera.zoom = zoom;
    }

    /// Returns the (min, max) world‑space corners that contain all worlds.
    fn world_bounds(&self, game: &mut Game) -> (Vec2, Vec2) {
        // Start with max possible values
        let mut min = vec2(f32::INFINITY, f32::INFINITY);
        let mut max = vec2(f32::NEG_INFINITY, f32::NEG_INFINITY);

        for world in &game.worlds {
            let tex = self.resolve_world_texture(world, &mut game.asset_manager);
            let w = tex.width() as f32;
            let h = tex.height() as f32;

            let pos = world.meta.position;
            let right = pos.x + w;
            let bottom = pos.y + h;

            if pos.x < min.x { min.x = pos.x; }
            if pos.y < min.y { min.y = pos.y; }
            if right > max.x { max.x = right; }
            if bottom > max.y { max.y = bottom; }
        }

        (min, max)
    }
}

/// A slice of all the modes.
static ALL_MODES: Lazy<&'static [GameEditorMode]> = Lazy::new(|| {
    Box::leak(Box::new(
        GameEditorMode::iter().collect::<Vec<_>>()
    ))
});

thread_local! {
    pub static EDIT_WORLD_RESULT: RefCell<Option<WorldEditResult>> = RefCell::new(None);
}

thread_local! {
    pub static DELETE_WORLD_RESULT: RefCell<Option<ConfirmPromptResult>> = RefCell::new(None);
}