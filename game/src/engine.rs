// game/src/engine.rs
use crate::transitions::transition_manager::TransitionManager;
use crate::scripting::script_system::ScriptSystem;
use crate::screen_space::render_screen_space;
use crate::diagnostics::DiagnosticsOverlay;
use crate::physics::physics_system::*;
use crate::game_instance::GameInstance;
use engine_core::prelude::*;
use bishop::prelude::*;
use bishop::BishopApp;
use std::cell::RefCell;
use std::rc::Rc;
use mlua::Lua;

pub struct Engine {
    /// Currently running instance of the game.
    pub game_instance: Rc<RefCell<GameInstance>>,
    /// Current state of the active game.
    pub game_state: GameState,
    /// Platform context for input/rendering.
    pub ctx: PlatformContext,
    /// Single Lua VM.
    pub lua: Lua,
    /// Camera manager for the game.
    pub camera_manager: CameraManager,
    /// Rendering system for the game.
    pub render_system: RenderSystem,
    /// Runtime diagnostics overlay (playtest only).
    pub diagnostics: DiagnosticsOverlay,
    /// Menu system for pause and overlay menus.
    pub menu: MenuManager,
    /// Whether the engine is running in playtest mode.
    pub is_playtest: bool,
    /// Accumulator for fixed timestep updates.
    pub accumulator: f32,
    /// Exponential moving average of frame time, used to smooth accumulator input.
    pub smoothed_dt: Option<f32>,
}

/// Represents the current state of the active game.
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Running,
    Paused,
}

impl BishopApp for Engine {
    async fn frame(&mut self, ctx: PlatformContext) {
        let raw_dt = ctx.borrow().get_frame_time();
        let smoothed = smooth_dt(&mut self.smoothed_dt, raw_dt, 0.9);
        let dt = snap_dt(smoothed);

        // Handle menu input first
        self.menu.handle_input(&mut *ctx.borrow_mut());
        self.update_game_state();

        if self.is_playtest {
            self.diagnostics.update(raw_dt);
            self.diagnostics.handle_input(&mut *ctx.borrow_mut());
        }

        if self.game_state == GameState::Running {
            self.accumulator = (self.accumulator + dt).min(MAX_ACCUM);

            while self.accumulator >= FIXED_DT {
                self.accumulator -= FIXED_DT;
                self.fixed_update(&mut *ctx.borrow_mut(), FIXED_DT);
            }

            self.update_async(raw_dt).await;
        }

        if self.is_playtest {
            self.update_diagnostics_metrics();
        }

        let alpha = (self.accumulator / FIXED_DT).clamp(0.0, 1.0);

        // Render game if visible
        if !self.menu.is_hiding_game() {
            self.render(&mut *ctx.borrow_mut(), alpha);
        } else {
            ctx.borrow_mut().clear_background(Color::BLACK);
        }

        // Render menus/ui on top
        self.menu.render(&mut *ctx.borrow_mut());
    }
}

impl Engine {
    /// Creates a new Engine with the given configuration.
    pub fn new(
        game_instance: Rc<RefCell<GameInstance>>,
        ctx: PlatformContext,
        lua: Lua,
        camera_manager: CameraManager,
        grid_size: f32,
        is_playtest: bool,
    ) -> Self {
        let mut menu = MenuManager::new();
        menu.set_action_handler(GameMenuHandler);

        Self {
            game_instance,
            game_state: GameState::Running,
            ctx,
            lua,
            camera_manager,
            render_system: RenderSystem::with_grid_size(grid_size),
            diagnostics: DiagnosticsOverlay::new(),
            menu,
            is_playtest,
            accumulator: 0.0,
            smoothed_dt: None,
        }
    }

    /// Resolves the current game state from all active systems.
    fn update_game_state(&mut self) {
        self.game_state = if self.menu.is_pausing_game() {
            GameState::Paused
        } else {
            GameState::Running
        };
    }

    pub fn fixed_update<C: BishopContext>(
        &mut self,
        ctx: &mut C,
        dt: f32
    ) {
        let mut game_instance = self.game_instance.borrow_mut();
        game_instance.store_previous_positions(&mut self.camera_manager);

        let game_ctx = game_instance.game.ctx_mut();
        let asset_manager = game_ctx.asset_manager;
        let ecs = game_ctx.ecs;

        let Some(current_room) = game_ctx.cur_world.current_room() else {
            return;
        };

        let grid_size = game_ctx.cur_world.grid_size;

        update_physics(
            asset_manager,
            ecs,
            current_room,
            dt,
            grid_size
        );

        self.camera_manager.update_active(
            ctx,
            ecs,
            current_room,
            game_ctx.cur_world.grid_size
        );
    }

    pub async fn update_async(&mut self, dt: f32) {
        {
            // Keep borrow_mut in this scope
            let mut game_instance = self.game_instance.borrow_mut();
            TransitionManager::handle_transitions(&mut game_instance);
            update_speech_timers(&mut game_instance.game.ecs, dt);

            let game_ctx = game_instance.game.ctx_mut();
            let asset_manager = game_ctx.asset_manager;
            let ecs = game_ctx.ecs;

            if let Some(current_room) = game_ctx.cur_world.current_room() {
                update_animation_sytem(ecs, asset_manager, dt, current_room.id).await;
            }

            // Load scripts in this scope TODO: make this part of run_scripts when scope is finalized
            let ctx = game_instance.game.ctx_mut();
            if let Err(e) = ScriptSystem::load_scripts(&self.lua, ctx.ecs, ctx.script_manager) {
                onscreen_error!("Error loading scripts: {}", e);
            }
        }

        // Run scripts outside borrow_mut scope
        if let Err(e) = ScriptSystem::run_scripts(dt, self) {
            onscreen_error!("Error running scripts: {}", e);
        }

        // Process menu events and emit to Lua
        self.process_menu_events();
    }

    pub fn render<C: BishopContext>(&mut self, ctx: &mut C, alpha: f32) {
        ctx.clear_background(Color::BLACK);

        let mut game_instance = self.game_instance.borrow_mut();
        let prev_positions = game_instance.prev_positions.clone();
        let dialogue_config = game_instance.game.dialogue_manager.config.clone();
        let game_ctx = game_instance.game.ctx_mut();

        let Some(current_room) = game_ctx.cur_world.current_room() else {
            return;
        };

        let grid_size = game_ctx.cur_world.grid_size;
        let render_cam = Camera2D {
            target: self.camera_manager.interpolated_target(alpha),
            zoom: self.camera_manager.active.camera.zoom,
            ..Default::default()
        };

        self.render_system.resize_for_camera(render_cam.zoom);
        self.render_system.begin_scene(ctx);

        render_room(
            ctx,
            game_ctx.ecs,
            current_room,
            game_ctx.asset_manager,
            &mut self.render_system,
            &render_cam,
            alpha,
            Some(&prev_positions),
            grid_size,
        );

        self.render_system.end_scene(ctx);
        self.render_system.present_game(ctx);

        render_screen_space(
            ctx,
            game_ctx.ecs,
            game_ctx.asset_manager,
            &dialogue_config,
            &render_cam,
            current_room.id,
            Some(&prev_positions),
            alpha,
            grid_size,
        );

        if self.is_playtest {
            self.diagnostics.draw(ctx);
        }
    }

    /// Update diagnostics metrics from game state.
    pub fn update_diagnostics_metrics(&mut self) {
        let game_instance = self.game_instance.borrow();
        let game = &game_instance.game;

        let entity_count = game.ecs.get_store::<Transform>().data.len();
        let texture_count = game.asset_manager.texture_count();
        let script_instances = game.script_manager.instances.len();
        let listener_count = game.script_manager.event_bus.listener_count();
        let script_id_count = game.script_manager.script_id_to_path.len();
        let sprite_id_count = game.asset_manager.sprite_id_to_path.len();
        let render_time_ms = self.render_system.render_time_ms;

        self.diagnostics.update_metrics(
            entity_count,
            texture_count,
            script_instances,
            listener_count,
            script_id_count,
            sprite_id_count,
            render_time_ms,
        );
    }

    /// Process menu events and emit them to Lua.
    fn process_menu_events(&mut self) {
        let events = drain_menu_events();
        if events.is_empty() {
            return;
        }

        let game_instance = self.game_instance.borrow();
        let event_bus = game_instance.game.script_manager.event_bus.clone();

        for action in events {
            let event_name = format!("menu:{}", action);
            event_bus.emit(event_name, mlua::Variadic::new());
        }
    }
}
