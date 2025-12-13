// game/src/engine.rs
use crate::scripting::script_system::run_scripts;
use crate::scripting::script_system::ScriptSystem;
use crate::scripting::script_system::load_scripts;
use crate::physics::physics_system::*;
use crate::game_state::GameState;
use engine_core::rendering::render_system::RenderSystem;
use engine_core::camera::camera_manager::CameraManager;
use engine_core::animation::animation_system::*;
use engine_core::ecs::component::CurrentRoom;
use engine_core::rendering::render_room::*;
use engine_core::ecs::component::Position;
use engine_core::onscreen_error;
use engine_core::constants::*;
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
    /// Camera that follows the player.
    pub camera_manager: CameraManager,
    /// Rendering system for the game.
    pub render_system: RenderSystem,
}

impl Engine {
    pub async fn run(&mut self) {
        let mut accumulator: f32 = 0.0;
        let mut cur_window_size = (screen_width() as u32, screen_height() as u32);

        // Main loop
        loop {
            // Time elapsed since last frame
            let frame_dt = get_frame_time();
            accumulator = (accumulator + frame_dt).min(MAX_ACCUM);
            
            // Fixed‑step physics
            while accumulator >= FIXED_DT {
                self.fixed_update(FIXED_DT);
                accumulator -= FIXED_DT;
            }

            // Per‑frame async work (input, animation, camera …)
            self.update_async(frame_dt).await;
        
            // Render with interpolation
            let alpha = accumulator / FIXED_DT;
            self.render(alpha, &mut cur_window_size);

            next_frame().await;
        }
    }

    pub fn fixed_update(&mut self, dt: f32,) {
        let mut game_state = self.game_state.borrow_mut();
        
        // Store the current positions for the next frame
        game_state.store_previous_positions(&mut self.camera_manager);

        // let mut game_state = self.game_state.borrow_mut();
        let game_ctx = game_state.game.ctx_mut();
        let world_ecs = game_ctx.cur_world_ecs;
        let current_room = game_ctx.cur_room;

        let player = world_ecs.get_player_entity();

        // If an entity exits the current room TODO: Decouple room transitions from physics
        if let Some((exiting_entity, target_id, new_pos)) = 
            update_physics(
                world_ecs, 
                current_room, 
                dt
            ) {
            // let new_room = game_state.game.current_world()
            //     .rooms
            //     .iter()
            //     .find(|r| r.id == target_id)
            //     .expect("Target room not found");

            // // Only update the game current room if the player exits
            // if exiting_entity == player {
            //     game_state.current_room = new_room.clone();
            // }

            let cur_room_mut = world_ecs.get_mut::<CurrentRoom>(exiting_entity).unwrap();
            // cur_room_mut.0 = new_room.id;

            let pos_mut = world_ecs.get_mut::<Position>(exiting_entity).unwrap();
            pos_mut.position = new_pos;
        }
    }

    pub async fn update_async(&mut self, dt: f32) {
        {
            // Keep borrow in this scope
            let mut game_state = self.game_state.borrow_mut();
            let game_ctx = game_state.game.ctx_mut();
            let asset_manager = game_ctx.asset_manager;
            let world_ecs = game_ctx.cur_world_ecs;
            let current_room = game_ctx.cur_room;
            
            let player_pos = world_ecs.get_player_position().position;
            
            // Update the camera
            self.camera_manager.update_active(
                world_ecs,
                current_room,
                player_pos,
            );
            
            update_animation_sytem(
                world_ecs,
                asset_manager,
                dt, 
                current_room.id,
            ).await;
            
            let ctx = game_state.game.ctx_mut();
            if let Err(e) = load_scripts(&self.lua, ctx.cur_world_ecs, ctx.script_manager) {
                onscreen_error!("Error loading scripts: {}", e);
            }
        }

        {
            // Scripts CAN'T have mutable state
            let game_state = self.game_state.borrow();
            let ctx = game_state.game.ctx();
            if let Err(e) = run_scripts(dt, ctx.cur_world_ecs, ctx.script_manager, &self.lua) {
                onscreen_error!("Error running scripts: {}", e);
            }
        }

        // Process queued commands
        ScriptSystem::process_commands(self);
    }

    pub fn render(&mut self, alpha: f32, cur_window_size: &mut (u32, u32)) {
        clear_background(BLACK);
        
        // Update the render system if the window is resized
        let cur_screen = (screen_width() as u32, screen_height() as u32);
        if cur_screen != *cur_window_size {
            self.render_system.resize(cur_screen.0, cur_screen.1);
            *cur_window_size = cur_screen;
        }
        
        let mut game_state = self.game_state.borrow_mut();
        let prev_positions = &game_state.prev_positions.clone();
        let game_ctx = game_state.game.ctx_mut();
        let asset_manager = game_ctx.asset_manager;
        let world_ecs = game_ctx.cur_world_ecs;
        let current_room = game_ctx.cur_room;

        let interpolated_target = lerp(
            self.camera_manager.previous_position.unwrap_or_default(),
            self.camera_manager.active.camera.target,
            alpha,
        );

        // Create a new interpolated camera
        let render_cam = Camera2D {
            target: interpolated_target,
            zoom: self.camera_manager.active.camera.zoom,
            ..Default::default()
        };

        render_room(
            world_ecs, 
            current_room, 
            asset_manager,
            &mut self.render_system,
            &render_cam,
            alpha,
            Some(prev_positions),
        );

        self.render_system.present_game();
    }
}

