// game/src/engine.rs
use crate::transitions::transition_manager::TransitionManager;
use crate::scripting::script_system::ScriptSystem;
use crate::screen_space::render_screen_space;
use crate::diagnostics::DiagnosticsOverlay;
use crate::game_global::set_menu_active;
use crate::game_instance::GameInstance;
use crate::physics::physics_system::*;
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
    pub menu_manager: MenuManager,
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
        self.menu_manager.handle_input(&mut *ctx.borrow_mut());
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
            self.diagnostics.update_from_game(
                &self.game_instance.borrow(), 
                self.render_system.render_time_ms
            );
        }

        let alpha = (self.accumulator / FIXED_DT).clamp(0.0, 1.0);

        // Render game if visible
        if !self.menu_manager.is_hiding_game() {
            self.render(&mut *ctx.borrow_mut(), alpha);
        } else {
            ctx.borrow_mut().clear_background(Color::BLACK);
        }

        // Render menus/ui on top
        self.render_menus(&ctx);
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
        let mut menu_manager = MenuManager::new();
        menu_manager.load_templates_from_disk();
        menu_manager.set_action_handler(GameMenuHandler);

        Self {
            game_instance,
            game_state: GameState::Running,
            ctx,
            lua,
            camera_manager,
            render_system: RenderSystem::with_grid_size(grid_size),
            diagnostics: DiagnosticsOverlay::new(),
            menu_manager,
            is_playtest,
            accumulator: 0.0,
            smoothed_dt: None,
        }
    }

    /// Resolves the current game state from all active systems.
    fn update_game_state(&mut self) {
        self.game_state = if self.menu_manager.is_pausing_game() {
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

        // Sync menu state for Lua scripts 
        // TODO: This should be decoupled from menus (does player movement need to be blocked for other reasons?)
        // Also reconsider global pattern here.
        set_menu_active(self.menu_manager.has_active_menu());

        // Run scripts outside borrow_mut scope
        if let Err(e) = ScriptSystem::run_scripts(dt, self) {
            onscreen_error!("Error running scripts: {}", e);
        }

        // Process menu events and emit to Lua
        self.game_instance.borrow().emit_menu_events();
    }

    pub fn render<C: BishopContext>(&mut self, ctx: &mut C, alpha: f32) {
        ctx.clear_background(Color::BLACK);

        let mut game_instance = self.game_instance.borrow_mut();
        let prev_positions = game_instance.prev_positions.clone();
        let text_config = game_instance.game.text_manager.config.clone();
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
            &text_config,
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

    fn render_menus(&mut self, ctx: &PlatformContext) {
        ctx.borrow_mut().flush_if_needed();
        let viewport = self.render_system.viewport_rect(&*ctx.borrow());
        self.menu_manager.set_viewport(viewport);
        let game_instance = self.game_instance.borrow();
        self.menu_manager.render(&mut *ctx.borrow_mut(), &game_instance.game.text_manager);
    }
}
