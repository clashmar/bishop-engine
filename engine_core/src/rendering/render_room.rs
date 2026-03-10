// engine_core/src/rendering/render_room.rs
// NOTE: Multi-pass rendering temporarily disabled while rewiring codebase.

use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};
use bishop::prelude::*;

/// Draws everything needed for the given room.
/// Currently uses simplified single-pass rendering.
pub fn render_room<C: BishopContext>(
    ctx: &mut C,
    ecs: &Ecs,
    room: &Room,
    asset_manager: &mut AssetManager,
    render_system: &mut RenderSystem,
    render_cam: &Camera2D,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
    grid_size: f32,
) {
    let render_start = std::time::Instant::now();

    // Cache the needed stores
    let sprite_store = ecs.get_store::<Sprite>();
    let frame_store = ecs.get_store::<CurrentFrame>();
    let transform_store = ecs.get_store::<Transform>();

    // Organize entities by layer
    let layer_map = collect_interpolated_layer_map(
        ecs,
        room,
        asset_manager,
        alpha,
        prev_positions,
        grid_size,
    );

    // Set up camera and clear background
    ctx.set_camera(render_cam);
    ctx.clear_background(Color::BLACK);

    // Draw tilemap first
    let tilemap = &room.current_variant().tilemap;
    tilemap.draw(ctx, asset_manager, room.position.into(), grid_size);

    // Draw all entities sorted by layer
    for (_z, (entities, _glows)) in layer_map {
        for (entity, pos) in entities {
            draw_entity(
                ctx,
                ecs,
                asset_manager,
                frame_store,
                sprite_store,
                transform_store,
                entity,
                pos,
                grid_size,
            );
        }

        // TODO: Re-enable multi-pass rendering
        // render_system.run_ambient_pass(ctx, room.darkness);
        // render_system.run_glow_pass(ctx, render_cam, glows, asset_manager);
        // render_system.run_undarkened_pass(ctx);
        // render_system.run_scene_pass(ctx);
    }

    // TODO: Re-enable multi-pass rendering
    // let lights = collect_lights(ecs, room, alpha, prev_positions);
    // render_system.run_spotlight_pass(ctx, render_cam, lights, room.darkness);
    // render_system.run_final_pass(ctx);

    render_system.render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;
}

fn draw_entity<C: BishopContext>(
    ctx: &mut C,
    ecs: &Ecs,
    asset_manager: &mut AssetManager,
    frame_store: &ComponentStore<CurrentFrame>,
    sprite_store: &ComponentStore<Sprite>,
    transform_store: &ComponentStore<Transform>,
    entity: Entity,
    pos: Vec2,
    grid_size: f32,
) {
    // If this is a player proxy, render using Player's visual components
    let visual_entity = if ecs.has::<PlayerProxy>(entity) {
        ecs.get_player_entity().unwrap_or(entity)
    } else {
        entity
    };

    let (width, height) = entity_dimensions(ecs, asset_manager, visual_entity, grid_size);

    // Get pivot from transform (default to BottomCenter)
    let pivot = transform_store
        .get(entity)
        .map(|t| t.pivot)
        .unwrap_or(Pivot::BottomCenter);

    // Calculate pivot-adjusted draw position
    let draw_base = pivot_adjusted_position(pos, Vec2::new(width, height), pivot);

    // Animate/Draw sprite (use visual_entity for sprite lookup)
    if let Some(cf) = frame_store.get(visual_entity) && asset_manager.contains(cf.sprite_id) {
        let tex = asset_manager.get_texture_from_id(cf.sprite_id);

        let frame_w = cf.frame_size.x;
        let frame_h = cf.frame_size.y;

        let src = Rect::new(
            cf.col as f32 * frame_w,
            cf.row as f32 * frame_h,
            frame_w,
            frame_h,
        );

        // Floor to be sure
        let draw_x = (draw_base.x + cf.offset.x).floor();
        let draw_y = (draw_base.y + cf.offset.y).floor();

        ctx.draw_texture_ex(
            tex,
            draw_x,
            draw_y,
            Color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(frame_w, frame_h)),
                source: Some(src),
                flip_x: cf.flip_x,
                ..Default::default()
            },
        );
        return;
    } else if let Some(sprite) = sprite_store.get(visual_entity) {
        // No animation
        if asset_manager.contains(sprite.sprite) {
            let tex = asset_manager.get_texture_from_id(sprite.sprite);
            ctx.draw_texture_ex(
                tex,
                draw_base.x,
                draw_base.y,
                Color::WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(width, height)),
                    ..Default::default()
                },
            );
            return;
        }
    }

    // Don't draw placeholders for these components
    if ecs.has_any::<(Light, Glow)>(visual_entity) {
        return;
    }

    // Fallback placeholder (no sprite or missing texture)
    draw_entity_placeholder(ctx, draw_base, grid_size);
}

/// Get the dimensions of an entity for rendering.
pub fn entity_dimensions(
    ecs: &Ecs,
    asset_manager: &AssetManager,
    entity: Entity,
    grid_size: f32,
) -> (f32, f32) {
    let from_anim = ecs
        .get_store::<CurrentFrame>()
        .get(entity)
        .map(|cf| (cf.frame_size.x, cf.frame_size.y));

    let from_sprite = || {
        ecs.get_store::<Sprite>()
            .get(entity)
            .and_then(|sprite| asset_manager.texture_size(sprite.sprite))
    };

    let from_glow = || {
        ecs.get_store::<Glow>()
            .get(entity)
            .and_then(|glow| asset_manager.texture_size(glow.sprite_id))
    };

    from_anim
        .or_else(from_sprite)
        .or_else(from_glow)
        .unwrap_or((grid_size, grid_size))
}

/// Draw a placeholder for an entity without a sprite.
pub fn draw_entity_placeholder<C: BishopContext>(
    ctx: &mut C,
    pos: Vec2,
    grid_size: f32
) {
    ctx.draw_rectangle(pos.x, pos.y, grid_size, grid_size, Color::GREEN);
}

/// Sorts entites by their z-layer, filters out entities that should not be
/// drawn and interpolates the draw positions. BTreeMap automatically sorts keys.
fn collect_interpolated_layer_map<'a>(
    ecs: &'a Ecs,
    room: &Room,
    asset_manager: &AssetManager,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
    grid_size: f32,
) -> BTreeMap<i32, (Vec<(Entity, Vec2)>, Vec<(&'a Glow, Vec2)>)> {
    let mut map: BTreeMap<i32, (Vec<(Entity, Vec2)>, Vec<(&Glow, Vec2)>)> = BTreeMap::new();

    let trans_store = ecs.get_store::<Transform>();
    let cam_store = ecs.get_store::<RoomCamera>();
    let room_store = ecs.get_store::<CurrentRoom>();
    let layer_store = ecs.get_store::<Layer>();
    let glow_store = ecs.get_store::<Glow>();

    for (entity, transform) in &trans_store.data {
        // Skip invisible entities
        if !transform.visible {
            continue;
        }

        // Skip camera
        if cam_store.get(*entity).is_some() {
            continue;
        }

        // Filter by current room
        if let Some(cr) = room_store.get(*entity) {
            if cr.0 != room.id {
                continue;
            }
        } else {
            continue;
        }

        // Interpolate the draw position
        let draw_pos =
            interpolate_draw_position(*entity, transform.position, alpha, prev_positions);

        // Default layer is 0 if missing
        let z = layer_store.get(*entity).map_or(0, |l| l.z);

        let entry = map.entry(z).or_default();
        entry.0.push((*entity, draw_pos));

        // If the entity also has a Glow component, apply pivot to glow position
        if let Some(glow) = glow_store.get(*entity) {
            let glow_size = asset_manager
                .texture_size(glow.sprite_id)
                .map(|(w, h)| Vec2::new(w, h))
                .unwrap_or(Vec2::new(grid_size, grid_size));

            let glow_draw_pos = pivot_adjusted_position(draw_pos, glow_size, transform.pivot);
            entry.1.push((glow, glow_draw_pos));
        }
    }

    // There always needs to be at least one layer otherwise nothing will be drawn
    if map.is_empty() {
        map.insert(0, (Vec::new(), Vec::new()));
    }

    map
}

// TODO: Re-enable for multi-pass rendering
// fn collect_lights(
//     ecs: &Ecs,
//     room: &Room,
//     alpha: f32,
//     prev_positions: Option<&HashMap<Entity, Vec2>>,
// ) -> Vec<(Vec2, Light)> { ... }

/// Returns the interpolated draw position or the current position.
fn interpolate_draw_position(
    entity: Entity,
    current_pos: Vec2,
    alpha: f32,
    prev_positions: Option<&HashMap<Entity, Vec2>>,
) -> Vec2 {
    if let Some(prev_map) = prev_positions {
        if let Some(prev_pos) = prev_map.get(&entity) {
            lerp_rounded(*prev_pos, current_pos, alpha)
        }
        else {
            current_pos
        }
    } else {
        current_pos
    }
}

/// Calculates draw position adjusted for pivot.
/// Returns the top-left corner where the texture should be drawn.
#[inline]
pub fn pivot_adjusted_position(entity_pos: Vec2, texture_size: Vec2, pivot: Pivot) -> Vec2 {
    let offset = pivot.as_normalized();
    vec2(
        entity_pos.x - texture_size.x * offset.x,
        entity_pos.y - texture_size.y * offset.y,
    )
}
