// game/src/engine.rs
use crate::diagnostics::DiagnosticsOverlay;
use crate::physics::physics_system::*;
use crate::game_state::GameState;
use crate::scripting::script_system::ScriptSystem;
use crate::transitions::transition_manager::TransitionManager;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::camera::camera_manager::CameraManager;
use engine_core::animation::animation_system::*;
use engine_core::rendering::render_room::*;
use engine_core::ecs::transform::Transform;
use engine_core::constants::*;
use engine_core::dialogue::*;
use macroquad::prelude::*;
use std::cell::RefCell;
use engine_core::*;
use std::rc::Rc;
use mlua::Lua;

pub struct Engine {
    /// Handle for the game.
    pub game_state: Rc<RefCell<GameState>>,
    /// Single Lua VM.
    pub lua: Lua,
    /// Camera manager for the game.
    pub camera_manager: CameraManager,
    /// Rendering system for the game.
    pub render_system: RenderSystem,
    /// Runtime diagnostics overlay (playtest only).
    pub diagnostics: DiagnosticsOverlay,
    /// Whether the engine is running in playtest mode.
    pub is_playtest: bool,
}

impl Engine {
    pub async fn run(&mut self) {
        let mut accumulator: f32 = 0.0;

        // Main loop
        loop {
            // Time elapsed since last frame
            let frame_dt = get_frame_time();
            accumulator = (accumulator + frame_dt).min(MAX_ACCUM);

            // Update diagnostics timing
            if self.is_playtest {
                self.diagnostics.update(frame_dt);
                self.diagnostics.handle_input();
            }

            while accumulator >= FIXED_DT {
                accumulator -= FIXED_DT;
                self.fixed_update(FIXED_DT);
            }

            // Per‑frame async work (input, animation)
            self.update_async(frame_dt).await;

            // Update diagnostics metrics before render
            if self.is_playtest {
                self.update_diagnostics_metrics();
            }

            // Render with interpolation
            let alpha = (accumulator / FIXED_DT).clamp(0.0, 1.0);
            self.render(alpha);

            next_frame().await;
        }
    }

    pub fn fixed_update(&mut self, dt: f32) {
        let mut game_state = self.game_state.borrow_mut();
        game_state.store_previous_positions(&mut self.camera_manager);

        let game_ctx = game_state.game.ctx_mut();
        let asset_manager = game_ctx.asset_manager;
        let ecs = game_ctx.ecs;

        let Some(current_room) = game_ctx.cur_world.current_room() else {
            return;
        };
        let grid_size = game_ctx.cur_world.grid_size;

        update_physics(asset_manager, ecs, current_room, dt, grid_size);

        self.camera_manager.update_active(ecs, current_room, game_ctx.cur_world.grid_size);
    }

    pub async fn update_async(&mut self, dt: f32) {
        {
            // Keep borrow_mut in this scope
            let mut game_state = self.game_state.borrow_mut();
            TransitionManager::handle_transitions(&mut game_state);
            update_speech_timers(&mut game_state.game.ecs, dt);

            let game_ctx = game_state.game.ctx_mut();
            let asset_manager = game_ctx.asset_manager;
            let ecs = game_ctx.ecs;

            if let Some(current_room) = game_ctx.cur_world.current_room() {
                update_animation_sytem(ecs, asset_manager, dt, current_room.id).await;
            }

            // Load scripts in this scope TODO: make this part of run_scripts when scope is finalized
            let ctx = game_state.game.ctx_mut();
            if let Err(e) = ScriptSystem::load_scripts(&self.lua, ctx.ecs, ctx.script_manager) {
                onscreen_error!("Error loading scripts: {}", e);
            }
        }

        // Run scripts outside borrow_mut scope
        if let Err(e) = ScriptSystem::run_scripts(dt, self) {
            onscreen_error!("Error running scripts: {}", e);
        }
    }

    pub fn render(&mut self, alpha: f32) {
        clear_background(BLACK);

        let mut game_state = self.game_state.borrow_mut();
        let prev_positions = game_state.prev_positions.clone();

        let game_ctx = game_state.game.ctx_mut();

        let asset_manager = game_ctx.asset_manager;
        let ecs = game_ctx.ecs;

        let Some(current_room) = game_ctx.cur_world.current_room() else {
            return;
        };

        let current_room_id = current_room.id;
        let grid_size = game_ctx.cur_world.grid_size;

        let target = self.camera_manager.interpolated_target(alpha);

        let render_cam = Camera2D {
            target: target,
            zoom: self.camera_manager.active.camera.zoom,
            ..Default::default()
        };

        self.render_system.resize_for_camera(render_cam.zoom);

        render_room(
            ecs,
            current_room,
            asset_manager,
            &mut self.render_system,
            &render_cam,
            alpha,
            Some(&prev_positions),
            grid_size,
        );

        self.render_system.present_game();

        // Collect speech bubble data
        let speech_bubbles = collect_speech_bubbles(
            ecs,
            asset_manager,
            current_room_id,
            alpha,
            Some(&prev_positions),
            grid_size,
        );

        // Render speech bubbles in screen space
        let dialogue_config = game_state.game.dialogue_manager.config.clone();
        render_speech_bubbles(&speech_bubbles, &dialogue_config, &render_cam, grid_size);
    

        // Draw diagnostics overlay after game rendering (playtest only)
        if self.is_playtest {
            draw_fps();
            self.diagnostics.draw();
        }
    }

    /// Update diagnostics metrics from game state.
    pub fn update_diagnostics_metrics(&mut self) {
        let game_state = self.game_state.borrow();
        let game = &game_state.game;

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
}
