// game/src/screen_space/mod.rs
use engine_core::prelude::*;
use bishop::prelude::*;
use std::collections::HashMap;

/// Renders all screen-space UI elements (speech bubbles, ui etc.).
pub fn render_screen_space<C: BishopContext>(
    ctx: &mut C,
    ecs: &Ecs,
    asset_manager: &AssetManager,
    dialogue_config: &DialogueConfig,
    render_cam: &Camera2D,
    room_id: RoomId,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
    alpha: f32,
    grid_size: f32,
) {
    render_speech(
        ctx, 
        ecs, 
        asset_manager, 
        dialogue_config, 
        render_cam, 
        room_id, 
        prev_positions, 
        alpha, 
        grid_size
    );
}

/// Renders speech bubbles in screen space above the game world.
fn render_speech<C: BishopContext>(
    ctx: &mut C,
    ecs: &Ecs,
    asset_manager: &AssetManager,
    dialogue_config: &DialogueConfig,
    render_cam: &Camera2D,
    room_id: RoomId,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
    alpha: f32,
    grid_size: f32,
) {
    let bubbles = collect_speech_bubbles(
        ecs, 
        asset_manager, 
        room_id, 
        alpha, 
        prev_positions, 
        grid_size
    );

    render_speech_bubbles(
        ctx, 
        &bubbles, 
        dialogue_config, 
        render_cam, 
        grid_size
    );
}
